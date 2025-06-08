use std::collections::HashMap;
use windows::{core::{Interface, PCSTR}, Win32::{Graphics::{Direct3D::{Fxc::D3DCompile, *}, Direct3D11::*, Dxgi::Common::*}}};

pub type Error = Box<dyn std::error::Error>;

unsafe fn translate_shader(wgsl: &str) -> Result<String, Error> {
	let module = naga::front::wgsl::parse_str(wgsl)?;
	
	let info = naga::valid::Validator::new(Default::default(), naga::valid::Capabilities::CLIP_DISTANCE | naga::valid::Capabilities::CULL_DISTANCE)
		.subgroup_stages(naga::valid::ShaderStages::all())
		.subgroup_operations(naga::valid::SubgroupOperationSet::all())
		.validate(&module)?;
	
	let options = naga::back::hlsl::Options {
		shader_model: naga::back::hlsl::ShaderModel::V5_0,
		..Default::default()
	};
	
	let mut buf = String::new();
	let mut writer = naga::back::hlsl::Writer::new(&mut buf, &options);
	writer.write(&module, &info, None)?;
	
	Ok(buf)
}

unsafe fn compile_shader(source: &str, entry: &str, target: &str) -> Result<Vec<u8>, Error> {
	let entry = format!("{entry}\0");
	let target = format!("{target}\0");
	
	let mut vs = None;
	let mut vs_err = None;
	D3DCompile(source.as_ptr() as _, source.len(), PCSTR("shader.hlsl\0".as_ptr()), None, None, PCSTR(entry.as_ptr()), PCSTR(target.as_ptr()), 0, 0, &mut vs, Some(&mut vs_err))
		.map_err(|_| {
			let err = vs_err.unwrap();
			std::str::from_utf8_unchecked(std::slice::from_raw_parts(err.GetBufferPointer() as _, err.GetBufferSize()))
		})?;
	
	let vs = vs.unwrap();
	Ok(std::slice::from_raw_parts(vs.GetBufferPointer() as _, vs.GetBufferSize()).to_vec())
}

// ------------------------------

#[derive(Debug)]
struct RenderTarget {
	#[allow(unused)] pub texture: ID3D11Texture2D,
	pub view: ID3D11RenderTargetView,
	pub sview: ID3D11ShaderResourceView,
}

impl RenderTarget {
	pub unsafe fn new(device: &ID3D11Device, w: u32, h: u32) -> Result<Self, Error> {
		let mut texture = None;
		device.CreateTexture2D(&D3D11_TEXTURE2D_DESC {
			Width: w,
			Height: h,
			MipLevels: 1,
			ArraySize: 1,
			Format: DXGI_FORMAT_R8G8B8A8_UNORM,
			SampleDesc: DXGI_SAMPLE_DESC {
				Count: 1,
				Quality: 0,
			},
			Usage: D3D11_USAGE_DEFAULT,
			BindFlags: (D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE).0 as u32,
			CPUAccessFlags: 0,
			MiscFlags: 0,
		}, None, Some(&mut texture))?;
		let texture = texture.unwrap();
		
		let mut view = None;
		device.CreateRenderTargetView(&texture, Some(&D3D11_RENDER_TARGET_VIEW_DESC {
			Format: DXGI_FORMAT_R8G8B8A8_UNORM,
			ViewDimension: D3D11_RTV_DIMENSION_TEXTURE2D,
			Anonymous: D3D11_RENDER_TARGET_VIEW_DESC_0 {
				Texture2D: D3D11_TEX2D_RTV {
					MipSlice: 0,
				},
			},
		}), Some(&mut view))?;
		let view = view.unwrap();
		
		let mut sview = None;
		device.CreateShaderResourceView(&texture, Some(&D3D11_SHADER_RESOURCE_VIEW_DESC {
			Format: DXGI_FORMAT_R8G8B8A8_UNORM,
			ViewDimension: D3D_SRV_DIMENSION_TEXTURE2D,
			Anonymous: D3D11_SHADER_RESOURCE_VIEW_DESC_0 {
				Texture2D: D3D11_TEX2D_SRV {
					MostDetailedMip: 0,
					MipLevels: 1,
				},
			},
		}), Some(&mut sview))?;
		let sview = sview.unwrap();
		
		Ok(Self {
			texture,
			view,
			sview,
		})
	}
	
	pub fn view_ptr(&self) -> *mut std::ffi::c_void {
		self.sview.as_raw()
	}
}

// ------------------------------

#[derive(Debug)]
struct Texture {
	pub texture: ID3D11Texture2D,
	pub sview: ID3D11ShaderResourceView,
	pub sampler: ID3D11SamplerState,
	pub data: Vec<u32>,
	pub w: u32,
}

impl Texture {
	pub unsafe fn new(device: &ID3D11Device, w: u32, h: u32, options: egui::TextureOptions) -> Result<Self, Error> {
		use egui::TextureFilter as F;
		
		let mut texture = None;
		device.CreateTexture2D(&D3D11_TEXTURE2D_DESC {
			Width: w,
			Height: h,
			MipLevels: 1,
			ArraySize: 1,
			Format: DXGI_FORMAT_R8G8B8A8_UNORM,
			SampleDesc: DXGI_SAMPLE_DESC {
				Count: 1,
				Quality: 0,
			},
			Usage: D3D11_USAGE_DYNAMIC,
			BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
			CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
			MiscFlags: 0,
		}, None, Some(&mut texture))?;
		let texture = texture.unwrap();
		
		let mut sview = None;
		device.CreateShaderResourceView(&texture, Some(&D3D11_SHADER_RESOURCE_VIEW_DESC {
			Format: DXGI_FORMAT_R8G8B8A8_UNORM,
			ViewDimension: D3D_SRV_DIMENSION_TEXTURE2D,
			Anonymous: D3D11_SHADER_RESOURCE_VIEW_DESC_0 {
				Texture2D: D3D11_TEX2D_SRV {
					MostDetailedMip: 0,
					MipLevels: 1,
				},
			},
		}), Some(&mut sview))?;
		let sview = sview.unwrap();
		
		let mut sampler = None;
		let wrap = match options.wrap_mode {
			egui::TextureWrapMode::ClampToEdge => D3D11_TEXTURE_ADDRESS_CLAMP,
			egui::TextureWrapMode::Repeat => D3D11_TEXTURE_ADDRESS_WRAP,
			egui::TextureWrapMode::MirroredRepeat => D3D11_TEXTURE_ADDRESS_MIRROR,
		};
		
		device.CreateSamplerState(&D3D11_SAMPLER_DESC {
			Filter: match (options.minification, options.magnification, options.mipmap_mode.unwrap_or(F::Nearest)) {
				(F::Nearest, F::Nearest, F::Nearest) => D3D11_FILTER_MIN_MAG_MIP_POINT,
				(F::Nearest, F::Nearest, F::Linear)  => D3D11_FILTER_MIN_MAG_POINT_MIP_LINEAR,
				(F::Nearest, F::Linear,  F::Nearest) => D3D11_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT,
				(F::Nearest, F::Linear,  F::Linear)  => D3D11_FILTER_MIN_POINT_MAG_MIP_LINEAR,
				(F::Linear,  F::Nearest, F::Nearest) => D3D11_FILTER_MIN_LINEAR_MAG_MIP_POINT,
				(F::Linear,  F::Nearest, F::Linear)  => D3D11_FILTER_MIN_LINEAR_MAG_POINT_MIP_LINEAR,
				(F::Linear,  F::Linear,  F::Nearest) => D3D11_FILTER_MIN_MAG_LINEAR_MIP_POINT,
				(F::Linear,  F::Linear,  F::Linear)  => D3D11_FILTER_MIN_MAG_MIP_LINEAR,
			},
			AddressU: wrap,
			AddressV: wrap,
			AddressW: wrap,
			..Default::default()
		}, Some(&mut sampler))?;
		let sampler = sampler.unwrap();
		
		Ok(Self {
			texture,
			sview,
			sampler,
			data: vec![0u32; (w * h) as usize],
			w,
		})
	}
	
	pub unsafe fn paint_region(&mut self, d3d11_ctx: &ID3D11DeviceContext, data: &[u32], sx: u32, sy: u32, w: u32, h: u32) {
		let mut pixels = data.iter();
		for y in sy..sy + h {
			for x in sx..sx + w {
				self.data[(y * self.w + x) as usize] = *pixels.next().unwrap();
			}
		}
		
		let mut data = D3D11_MAPPED_SUBRESOURCE::default();
		d3d11_ctx.Map(&self.texture, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut data)).unwrap();
		core::ptr::copy_nonoverlapping(self.data.as_ptr(), data.pData as _, self.data.len());
	}
}

// ------------------------------

#[derive(Debug)]
pub struct Renderer {
	d3d11_ctx: ID3D11DeviceContext,
	
	rt: RenderTarget,
	vs: ID3D11VertexShader,
	ps: ID3D11PixelShader,
	raster: ID3D11RasterizerState,
	blend: ID3D11BlendState,
	layout: ID3D11InputLayout,
	
	egui_ctx: egui::Context,
	start: std::time::Instant,
	textures: HashMap<u64, Texture>,
	last_io: crate::Io,
	last_io_keys: [bool; 256],
	last_cursor_icon: egui::CursorIcon,
	vertex_buffer: ID3D11Buffer,
	index_buffer: ID3D11Buffer,
}

impl Renderer {
	pub fn new(device_ptr: usize) -> Result<Self, Error> {unsafe{
		let device = &(device_ptr as _);
		let device = ID3D11Device::from_raw_borrowed(device).ok_or("Failed borrowing device")?;
		
		let mut context = None;
		device.CreateDeferredContext(0, Some(&mut context))?;
		
		const SHADER: &str = include_str!("egui.wgsl");
		let shader = translate_shader(SHADER)?;
		// crate::log!("{shader}");
		
		let mut vs = None;
		let vs_bytecode = compile_shader(&shader, "vs_main", "vs_5_0")?;
		device.CreateVertexShader(&vs_bytecode, None, Some(&mut vs))?;
		
		let mut ps = None;
		device.CreatePixelShader(&compile_shader(&shader, "fs_main", "ps_5_0")?, None, Some(&mut ps))?;
		
		let mut raster = None;
		device.CreateRasterizerState(&D3D11_RASTERIZER_DESC {
			FillMode: D3D11_FILL_SOLID,
			CullMode: D3D11_CULL_NONE,
			ScissorEnable: windows::core::BOOL(1),
			..Default::default()
		}, Some(&mut raster))?;
		
		let mut blend = None;
		device.CreateBlendState(&D3D11_BLEND_DESC {
			RenderTarget: [
				D3D11_RENDER_TARGET_BLEND_DESC {
					BlendEnable: windows::core::BOOL(1),
					SrcBlend: D3D11_BLEND_ONE,
					DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
					BlendOp: D3D11_BLEND_OP_ADD,
					SrcBlendAlpha: D3D11_BLEND_INV_DEST_ALPHA,
					DestBlendAlpha: D3D11_BLEND_ONE,
					BlendOpAlpha: D3D11_BLEND_OP_ADD,
					RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as _,
				},
				Default::default(),
				Default::default(),
				Default::default(),
				Default::default(),
				Default::default(),
				Default::default(),
				Default::default()
			],
			..Default::default()
		}, Some(&mut blend))?;
		
		fn input(index: u32, format: DXGI_FORMAT, slot: u32, offset: u32, class: D3D11_INPUT_CLASSIFICATION, step: u32) -> D3D11_INPUT_ELEMENT_DESC {
			D3D11_INPUT_ELEMENT_DESC {
				SemanticName: PCSTR("LOC\0".as_ptr()),
				SemanticIndex: index,
				Format: format,
				InputSlot: slot,
				AlignedByteOffset: offset,
				InputSlotClass: class,
				InstanceDataStepRate: step,
			}
		}
		
		let mut layout = None;
		device.CreateInputLayout(&[
			input(0, DXGI_FORMAT_R32G32_FLOAT, 0, 0, D3D11_INPUT_PER_VERTEX_DATA, 0),
			input(1, DXGI_FORMAT_R32G32_FLOAT, 0, D3D11_APPEND_ALIGNED_ELEMENT, D3D11_INPUT_PER_VERTEX_DATA, 0),
			input(2, DXGI_FORMAT_R32G32B32A32_FLOAT, 0, D3D11_APPEND_ALIGNED_ELEMENT, D3D11_INPUT_PER_VERTEX_DATA, 0),
			// input(2, DXGI_FORMAT_R8G8B8A8_UNORM_SRGB, 0, D3D11_APPEND_ALIGNED_ELEMENT, D3D11_INPUT_PER_VERTEX_DATA, 0),
		], &vs_bytecode, Some(&mut layout))?;
		
		Ok(Self {
			// d3d11
			d3d11_ctx: context.unwrap(),
			
			rt: RenderTarget::new(&device, 256, 256)?,
			vs: vs.unwrap(),
			ps: ps.unwrap(),
			raster: raster.unwrap(),
			blend: blend.unwrap(),
			layout: layout.unwrap(),
			
			// egui
			egui_ctx: Default::default(),
			start: std::time::Instant::now(),
			textures: HashMap::new(),
			last_io: Default::default(),
			last_io_keys: [false; 256],
			last_cursor_icon: Default::default(),
			
			// todo: create when drawing, tried it and it spazzes out the game rendering (looked neat tho)
			vertex_buffer: {
				let mut buffer = None;
				device.CreateBuffer(&D3D11_BUFFER_DESC {
					ByteWidth: 65536 * 8 * 4, // 65536 vertices
					Usage: D3D11_USAGE_DYNAMIC,
					BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
					CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
					..Default::default()
				}, None, Some(&mut buffer)).unwrap();
				buffer.unwrap()
			},
			index_buffer: {
				let mut buffer = None;
				device.CreateBuffer(&D3D11_BUFFER_DESC {
					ByteWidth: 65536 * 4, // 65536 indices
					Usage: D3D11_USAGE_DYNAMIC,
					BindFlags: D3D11_BIND_INDEX_BUFFER.0 as u32,
					CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
					..Default::default()
				}, None, Some(&mut buffer)).unwrap();
				buffer.unwrap()
			},
		})
	}}
	
	pub fn draw(&mut self, device_ptr: usize, io: crate::Io, draw: impl FnMut(&egui::Context)) -> Result<usize, Error> {unsafe{
		let device = &(device_ptr as _);
		let device = ID3D11Device::from_raw_borrowed(device).ok_or("Failed borrowing device")?;
		
		let (w, h) = (io.width, io.height);
		if (self.last_io.width != w || self.last_io.height != h) && w > 16.0 && h > 16.0 {
			// crate::log!("resizing rendertarget to {w}x{h}");
			self.rt = RenderTarget::new(device, w as u32, h as u32)?;
		}
		
		let primitives = self.handle_egui(device, io.clone(), draw)?;
		
		self.d3d11_ctx.ClearRenderTargetView(&self.rt.view, &[0.0, 0.0, 0.0, 0.0]);
		self.d3d11_ctx.RSSetViewports(Some(&[D3D11_VIEWPORT {
			TopLeftX: 0.0,
			TopLeftY: 0.0,
			Width: w,
			Height: h,
			MinDepth: 0.0,
			MaxDepth: 1.0,
		}]));
		self.d3d11_ctx.OMSetRenderTargets(Some(&[Some(self.rt.view.clone())]), None);
		
		self.d3d11_ctx.VSSetShader(Some(&self.vs), None);
		self.d3d11_ctx.PSSetShader(Some(&self.ps), None);
		self.d3d11_ctx.RSSetState(&self.raster);
		self.d3d11_ctx.OMSetBlendState(&self.blend, Some(&[0.0; 4]), 0xFFFFFFFF);
		self.d3d11_ctx.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
		self.d3d11_ctx.IASetInputLayout(Some(&self.layout));
		
		for prim in primitives {
			self.d3d11_ctx.RSSetScissorRects(Some(&[windows::Win32::Foundation::RECT {
				left: prim.clip_rect.left() as i32,
				top: prim.clip_rect.top() as i32,
				right: prim.clip_rect.right() as i32,
				bottom: prim.clip_rect.bottom() as i32,
			}]));
			
			match prim.primitive {
				egui::epaint::Primitive::Callback(_) => {}
				egui::epaint::Primitive::Mesh(mesh) => {
					if let egui::TextureId::Managed(id) = mesh.texture_id {
						if let Some(tex) = self.textures.get(&id) {
							self.d3d11_ctx.PSSetSamplers(0, Some(&[Some(tex.sampler.clone())]));
							self.d3d11_ctx.PSSetShaderResources(0, Some(&[Some(tex.sview.clone())]));
						}
					}
					
					let vertices = mesh.vertices.into_iter().map(|v| [
						v.pos.x / io.width * 2.0 - 1.0, 1.0 - v.pos.y / io.height * 2.0,
						v.uv.x, v.uv.y,
						v.color.r() as f32 / 255.0, v.color.g() as f32 / 255.0, v.color.b() as f32 / 255.0, v.color.a() as f32 / 255.0,
					]).collect::<Vec<_>>();
					let mut data = D3D11_MAPPED_SUBRESOURCE::default();
					self.d3d11_ctx.Map(&self.vertex_buffer, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut data))?;
					core::ptr::copy_nonoverlapping(vertices.as_ptr(), data.pData as _, vertices.len());
					self.d3d11_ctx.IASetVertexBuffers(0, 1, Some(&Some(self.vertex_buffer.clone())), Some(&(8 * 4)), Some(&0));
					
					let mut data = D3D11_MAPPED_SUBRESOURCE::default();
					self.d3d11_ctx.Map(&self.index_buffer, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut data))?;
					core::ptr::copy_nonoverlapping(mesh.indices.as_ptr(), data.pData as _, mesh.indices.len());
					self.d3d11_ctx.IASetIndexBuffer(&self.index_buffer, DXGI_FORMAT_R32_UINT, 0);
					
					self.d3d11_ctx.DrawIndexed(mesh.indices.len() as u32, 0, 0);
				}
			}
		}
		
		let mut cmdlist = None;
		self.d3d11_ctx.FinishCommandList(true, Some(&mut cmdlist))?;
		let cmdlist = cmdlist.unwrap();
		device.GetImmediateContext()?.ExecuteCommandList(Some(&cmdlist), true);
		self.d3d11_ctx.ClearState();
		
		Ok(self.rt.view_ptr() as _)
	}}
	
	pub fn egui_ctx(&self) -> egui::Context {
		self.egui_ctx.clone()
	}
	
	fn handle_egui(&mut self, device: &ID3D11Device, io: crate::Io, draw: impl FnMut(&egui::Context)) -> Result<Vec<egui::ClippedPrimitive>, Error> {
		let modifiers = egui::Modifiers {
			alt: io.mods & 0b001 != 0,
			ctrl: io.mods & 0b010 != 0,
			shift: io.mods & 0b100 != 0,
			mac_cmd: false,
			command: io.mods & 0b010 != 0,
		};
		
		let mut events = Vec::new();
		if self.last_io.mouse_x != io.mouse_x || self.last_io.mouse_y != io.mouse_y {
			events.push(egui::Event::PointerMoved(egui::pos2(io.mouse_x, io.mouse_y)));
		}
		
		if self.last_io.wheel_x != io.wheel_x || self.last_io.wheel_y != io.wheel_y {
			events.push(egui::Event::MouseWheel {
				unit: egui::MouseWheelUnit::Line,
				delta: egui::vec2(io.wheel_x, io.wheel_y),
				modifiers,
			});
		}
		
		for i in 0..5 {
			let mask = 0b00001000 << i;
			if self.last_io.mods & mask != io.mods & mask {
				events.push(egui::Event::PointerButton {
					pos: egui::pos2(io.mouse_x, io.mouse_y),
					button: unsafe{std::mem::transmute(i as u8)},
					pressed: io.mods & mask != 0,
					modifiers,
				});
			}
		}
		
		if io.input_buf_ptr != 0 {
			let inputs = unsafe{std::slice::from_raw_parts(io.input_buf_ptr as *const u16, io.input_buf_len)}.to_vec();
			for input in char::decode_utf16(inputs) {
				let Ok(c) = input else {continue};
				if !is_printable_char(c) {continue};
				
				events.push(egui::Event::Text(c.to_string()));
			}
		}
		
		if self.last_io.mods & 0b1_00000000 != io.mods & 0b1_00000000 {
			events.push(egui::Event::WindowFocused(io.mods & 0b1_00000000 != 0));
			
			if io.mods & 0b1_00000000 == 0 {
				events.push(egui::Event::PointerGone);
			}
		}
		
		let io_keys = get_keyboard_state();
		for (keycode, (last, now)) in self.last_io_keys.iter().zip(io_keys.iter()).enumerate() {
			if *last != *now {
				let Some(key) = convert_key(keycode as u8) else {continue};
				
				events.push(egui::Event::Key {
					key,
					physical_key: None,
					pressed: *now,
					repeat: false,
					modifiers,
				});
			}
		}
		
		if modifiers.command && io_keys[0x43] && !self.last_io_keys[0x43] {
			events.push(egui::Event::Copy);
		}
		
		if modifiers.command && io_keys[0x58] && !self.last_io_keys[0x58] {
			events.push(egui::Event::Cut);
		}
		
		if modifiers.command && io_keys[0x56] && !self.last_io_keys[0x56] {
			if let Ok(text) = clipboard_win::get_clipboard_string() {
				events.push(egui::Event::Paste(text));
			}
		}
		
		let input = egui::RawInput {
			screen_rect: Some(egui::Rect {
				min: egui::pos2(0.0, 0.0),
				max: egui::pos2(io.width, io.height),
			}),
			max_texture_side: Some(2048),
			time: Some(std::time::Instant::now().duration_since(self.start).as_secs_f64()),
			predicted_dt: 0.0, // egui calculates dt as we provide time
			modifiers,
			events,
			focused: io.mods & 0b1_00000000 != 0,
			..Default::default()
		};
		
		self.egui_ctx.options_mut(|v| v.line_scroll_speed = 80.0);
		self.egui_ctx.all_styles_mut(|v| {
			v.url_in_tooltip = true;
			v.visuals.slider_trailing_fill = true;
		});
		let out = self.egui_ctx.run(input, draw);
		
		if io.mods & 0b1_00000000 != 0 && self.egui_ctx.wants_keyboard_input() {
			unsafe{*(io.set_keyboard_focus as *mut u8) = 1};
		}
		
		if !out.platform_output.copied_text.is_empty() {
			_ = clipboard_win::set_clipboard_string(&out.platform_output.copied_text);
		}
		
		if let Some(hypr) = out.platform_output.open_url {
			if hypr.url.starts_with("http://") || hypr.url.starts_with("https://") {
				_ = opener::open(&hypr.url);
			}
		}
		
		if self.last_cursor_icon != out.platform_output.cursor_icon {
			use windows::Win32::UI::WindowsAndMessaging::*;
			use egui::CursorIcon as I;
			
			// https://github.com/rust-windowing/winit/blob/master/winit-win32/src/util.rs#L168
			let icon = match out.platform_output.cursor_icon {
				// I::None => todo!(),
				I::Default => IDC_ARROW,
				I::PointingHand => IDC_HAND,
				I::Crosshair => IDC_CROSS,
				I::Text | I::VerticalText => IDC_IBEAM,
				I::NotAllowed | I::NoDrop => IDC_NO,
				I::Grab | I::Grabbing | I::Move | I::AllScroll => IDC_SIZEALL,
				I::ResizeEast | I::ResizeWest | I::ResizeHorizontal | I::ResizeColumn => IDC_SIZEWE,
				I::ResizeNorth | I::ResizeSouth | I::ResizeVertical | I::ResizeRow => IDC_SIZENS,
				I::ResizeNorthEast | I::ResizeSouthWest | I::ResizeNeSw => IDC_SIZENESW,
				I::ResizeNorthWest | I::ResizeSouthEast | I::ResizeNwSe => IDC_SIZENWSE,
				I::Wait => IDC_WAIT,
				I::Progress => IDC_APPSTARTING,
				I::Help => IDC_HELP,
				_ => IDC_ARROW
			};
			
			unsafe {
				let cursor = LoadCursorW(None, icon);
				SetCursor(cursor.ok());
			}
		}
		
		for (id, delta) in out.textures_delta.set {
			let egui::TextureId::Managed(id) = id else {continue};
			let w = delta.image.width() as u32;
			let h = delta.image.height() as u32;
			
			if delta.is_whole() {
				self.textures.remove(&id);
				self.textures.insert(id, unsafe{Texture::new(device, w, h, delta.options).unwrap()});
			}
			
			let texture = self.textures.get_mut(&id).unwrap();
			
			let sx = delta.pos.map_or(0, |v| v[0] as u32);
			let sy = delta.pos.map_or(0, |v| v[1] as u32);
			// crate::log(aetherment::LogType::Log, &format!("updating texture[{id}] {sx},{sy} {w}x{h}; whole: {}", delta.is_whole()));
			
			let pixels = match delta.image {
				egui::ImageData::Color(img) => img.pixels.clone(),
				egui::ImageData::Font(img) => img.srgba_pixels(None).collect::<Vec<_>>(),
			};
			
			unsafe{texture.paint_region(&self.d3d11_ctx, std::mem::transmute::<_, &[u32]>(&pixels[..]), sx, sy, w, h)}
		}
		
		for id in out.textures_delta.free {
			let egui::TextureId::Managed(id) = id else {continue};
			self.textures.remove(&id);
		}
		
		self.last_io = io;
		self.last_io_keys = io_keys;
		self.last_cursor_icon = out.platform_output.cursor_icon;
		
		Ok(self.egui_ctx.tessellate(out.shapes, 1.0))
	}
}

// ------------------------------

// https://github.com/emilk/egui/blob/6d04140736ddfbd1e406195de6804f57f1406321/crates/egui-winit/src/lib.rs#L1038
/// Winit sends special keys (backspace, delete, F1, …) as characters.
/// Ignore those.
/// We also ignore '\r', '\n', '\t'.
/// Newlines are handled by the `Key::Enter` event.
fn is_printable_char(chr: char) -> bool {
	let is_in_private_use_area = '\u{e000}' <= chr && chr <= '\u{f8ff}'
		|| '\u{f0000}' <= chr && chr <= '\u{ffffd}'
		|| '\u{100000}' <= chr && chr <= '\u{10fffd}';
	
	!is_in_private_use_area && !chr.is_ascii_control()
}

extern "C" {fn GetKeyboardState(key_states: *mut [u8; 256]);}
fn get_keyboard_state() -> [bool; 256] {
	let mut key_states = [0; 256];
	unsafe{GetKeyboardState(&mut key_states)};
	let key_states: [bool; 256] = key_states.into_iter().map(|v| v & 0x80 != 0).collect::<Vec<bool>>().try_into().unwrap();
	key_states
}

fn convert_key(key: u8) -> Option<egui::Key> {
	use egui::Key::*;
	match key {
		0x08 => Some(Backspace),
		0x09 => Some(Tab),
		0x0D => Some(Enter),
		0x18 => Some(Escape),
		0x20 => Some(Space),
		
		0x21 => Some(PageUp),
		0x22 => Some(PageDown),
		0x24 => Some(Home),
		0x23 => Some(End),
		0x2D => Some(Insert),
		0x2E => Some(Delete),
		
		0x25 => Some(ArrowLeft),
		0x26 => Some(ArrowUp),
		0x27 => Some(ArrowRight),
		0x28 => Some(ArrowDown),
		
		0x30 | 0x60 => Some(Num0),
		0x31 | 0x61 => Some(Num1),
		0x32 | 0x62 => Some(Num2),
		0x33 | 0x63 => Some(Num3),
		0x34 | 0x64 => Some(Num4),
		0x35 | 0x65 => Some(Num5),
		0x36 | 0x66 => Some(Num6),
		0x37 | 0x67 => Some(Num7),
		0x38 | 0x68 => Some(Num8),
		0x39 | 0x69 => Some(Num9),
		
		0x41 => Some(A),
		0x42 => Some(B),
		0x43 => Some(C),
		0x44 => Some(D),
		0x45 => Some(E),
		0x46 => Some(F),
		0x47 => Some(G),
		0x48 => Some(H),
		0x49 => Some(I),
		0x4A => Some(J),
		0x4B => Some(K),
		0x4C => Some(L),
		0x4D => Some(M),
		0x4E => Some(N),
		0x4F => Some(O),
		0x50 => Some(P),
		0x51 => Some(Q),
		0x52 => Some(R),
		0x53 => Some(S),
		0x54 => Some(T),
		0x55 => Some(U),
		0x56 => Some(V),
		0x57 => Some(W),
		0x58 => Some(X),
		0x59 => Some(Y),
		0x5A => Some(Z),
		
		0x70 => Some(F1),
		0x71 => Some(F2),
		0x72 => Some(F3),
		0x73 => Some(F4),
		0x74 => Some(F5),
		0x75 => Some(F6),
		0x76 => Some(F7),
		0x77 => Some(F8),
		0x78 => Some(F9),
		0x79 => Some(F10),
		0x7A => Some(F11),
		0x7B => Some(F12),
		0x7C => Some(F13),
		0x7D => Some(F14),
		0x7E => Some(F15),
		0x7F => Some(F16),
		0x80 => Some(F17),
		0x81 => Some(F18),
		0x82 => Some(F19),
		0x83 => Some(F20),
		0x84 => Some(F21),
		0x85 => Some(F22),
		0x86 => Some(F23),
		0x87 => Some(F24),
		
		0xBA => Some(Semicolon),
		0xBB => Some(Plus),
		0xBC => Some(Comma),
		0xBD => Some(Minus),
		0xBE => Some(Period),
		0xBF => Some(Slash),
		0xC0 => Some(Backtick),
		0xDB => Some(OpenBracket),
		0xDC => Some(Pipe),
		0xDD => Some(CloseBracket),
		0xDE => Some(Quote),
		0xDF => Some(Minus),
		
		_ => None,
	}
}