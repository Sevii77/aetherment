use std::{collections::HashMap, rc::Rc};
use windows::{core::{Interface, PCSTR}, Win32::Graphics::{Direct3D::{Fxc::D3DCompile, *}, Direct3D11::*, Dxgi::Common::*}};

pub struct D3d11Renderer {
	device: &'static ID3D11Device,
	context: Rc<ID3D11DeviceContext>,
	texture_register: Box<dyn Fn(&super::Texture) -> u64>,
	
	depth_stencil_state: ID3D11DepthStencilState,
	rasterizer_state: ID3D11RasterizerState,
}

impl D3d11Renderer {
	pub fn new(device_ptr: usize, texture_register: Box<dyn Fn(&super::Texture) -> u64>) -> Self {
		let device = &(device_ptr as _);
		let device = unsafe{std::mem::transmute::<&*mut std::ffi::c_void, &'static *mut std::ffi::c_void>(device)};
		let device = unsafe{ID3D11Device::from_raw_borrowed(device).ok_or("Failed borrowing device").unwrap()};
		
		let mut context = None;
		unsafe{device.CreateDeferredContext(0, Some(&mut context)).unwrap()};
		
		let mut depth_stencil_state = None;
		unsafe{device.CreateDepthStencilState(&D3D11_DEPTH_STENCIL_DESC {
			DepthEnable: windows::core::BOOL(1),
			DepthWriteMask: D3D11_DEPTH_WRITE_MASK_ALL,
			DepthFunc: D3D11_COMPARISON_LESS,
			StencilEnable: windows::core::BOOL(0),
			..Default::default()
		}, Some(&mut depth_stencil_state)).unwrap()};
		
		let mut rasterizer_state = None;
		unsafe{device.CreateRasterizerState(&D3D11_RASTERIZER_DESC {
			FillMode: D3D11_FILL_SOLID,
			CullMode: D3D11_CULL_FRONT,
			ScissorEnable: windows::core::BOOL(0),
			..Default::default()
		}, Some(&mut rasterizer_state)).unwrap()};
		
		Self {
			device,
			context: std::rc::Rc::new(context.unwrap()),
			texture_register,
			
			depth_stencil_state: depth_stencil_state.unwrap(),
			rasterizer_state: rasterizer_state.unwrap(),
		}
	}
	
	pub fn update_device(&mut self, device_ptr: usize) {
		let device = &(device_ptr as _);
		let device = unsafe{std::mem::transmute::<&*mut std::ffi::c_void, &'static *mut std::ffi::c_void>(device)};
		self.device = unsafe{ID3D11Device::from_raw_borrowed(device).ok_or("Failed borrowing device").unwrap()};
	}
}

impl super::RendererInner for D3d11Renderer {
	fn create_material(&self, shader: &str, binds: &[super::MaterialBind]) -> super::Material {
		let hlsl = {
			let module = naga::front::wgsl::parse_str(shader).unwrap();
			
			let info = naga::valid::Validator::new(Default::default(), naga::valid::Capabilities::CLIP_DISTANCE | naga::valid::Capabilities::CULL_DISTANCE)
				.subgroup_stages(naga::valid::ShaderStages::all())
				.subgroup_operations(naga::valid::SubgroupOperationSet::all())
				.validate(&module)
				.unwrap();
			
			let options = naga::back::hlsl::Options {
				shader_model: naga::back::hlsl::ShaderModel::V5_0,
				..Default::default()
			};
			
			let mut buf = String::new();
			let mut writer = naga::back::hlsl::Writer::new(&mut buf, &options);
			writer.write(&module, &info, None).unwrap();
			
			buf
		};
		
		let mut vs = None;
		let vs_bytecode = unsafe{compile_shader(&hlsl, "vs_main", "vs_5_0").unwrap()};
		unsafe{self.device.CreateVertexShader(&vs_bytecode, None, Some(&mut vs)).unwrap()};
		
		let mut ps = None;
		unsafe{self.device.CreatePixelShader(&compile_shader(&hlsl, "fs_main", "ps_5_0").unwrap(), None, Some(&mut ps))}.unwrap();
		
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
		unsafe{self.device.CreateInputLayout(&[
			input(0, DXGI_FORMAT_R32G32_FLOAT, 0, 0, D3D11_INPUT_PER_VERTEX_DATA, 0),
			input(1, DXGI_FORMAT_R32G32B32_FLOAT, 0, D3D11_APPEND_ALIGNED_ELEMENT, D3D11_INPUT_PER_VERTEX_DATA, 0),
			input(2, DXGI_FORMAT_R32G32B32_FLOAT, 0, D3D11_APPEND_ALIGNED_ELEMENT, D3D11_INPUT_PER_VERTEX_DATA, 0),
			input(3, DXGI_FORMAT_R32G32B32A32_FLOAT, 0, D3D11_APPEND_ALIGNED_ELEMENT, D3D11_INPUT_PER_VERTEX_DATA, 0),
			input(4, DXGI_FORMAT_R32G32B32A32_FLOAT, 0, D3D11_APPEND_ALIGNED_ELEMENT, D3D11_INPUT_PER_VERTEX_DATA, 0),
		], &vs_bytecode, Some(&mut layout)).unwrap()};
		
		Rc::new(D3d11Material {
			binds: binds.to_vec(),
			vs: vs.unwrap(),
			ps: ps.unwrap(),
			layout: layout.unwrap(),
		})
	}
	
	fn create_texture(&self, width: u32, height: u32, format: super::TextureFormat, usage: super::TextureUsage) -> super::Texture {
		let format = match format {
			super::TextureFormat::Rgba8Unorm => DXGI_FORMAT_R8G8B8A8_UNORM,
			super::TextureFormat::Rgba8UnormSrgb => DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
			super::TextureFormat::Depth32Float => DXGI_FORMAT_D32_FLOAT,
		};
		
		let mut texture = None;
		unsafe{self.device.CreateTexture2D(&D3D11_TEXTURE2D_DESC {
			Width: width,
			Height: height,
			MipLevels: 1,
			ArraySize: 1,
			Format: format,
			SampleDesc: DXGI_SAMPLE_DESC {
				Count: 1,
				Quality: 0,
			},
			Usage: D3D11_USAGE_DYNAMIC,
			BindFlags:
				if usage.contains(super::TextureUsage::TEXTURE_BINDING) {D3D11_BIND_SHADER_RESOURCE.0 as u32} else {0} |
				if usage.contains(super::TextureUsage::RENDER_TARGET) {D3D11_BIND_RENDER_TARGET.0 as u32} else {0} |
				if usage.contains(super::TextureUsage::DEPTH_STENCIL) {D3D11_BIND_DEPTH_STENCIL.0 as u32} else {0},
			CPUAccessFlags:
				if usage.contains(super::TextureUsage::COPY_DST) {D3D11_CPU_ACCESS_WRITE.0 as u32} else {0} |
				if usage.contains(super::TextureUsage::COPY_SRC) {D3D11_CPU_ACCESS_READ.0 as u32} else {0},
			MiscFlags: 0,
		}, None, Some(&mut texture)).unwrap()};
		let texture = texture.unwrap();
		
		let mut view = None;
		if usage.contains(super::TextureUsage::TEXTURE_BINDING) {
			unsafe{self.device.CreateShaderResourceView(&texture, Some(&D3D11_SHADER_RESOURCE_VIEW_DESC {
				Format: format,
				ViewDimension: D3D_SRV_DIMENSION_TEXTURE2D,
				Anonymous: D3D11_SHADER_RESOURCE_VIEW_DESC_0 {
					Texture2D: D3D11_TEX2D_SRV {
						MostDetailedMip: 0,
						MipLevels: 1,
					},
				},
			}), Some(&mut view)).unwrap()};
		}
		
		let mut view_rt = None;
		if usage.contains(super::TextureUsage::RENDER_TARGET) {
			unsafe{self.device.CreateRenderTargetView(&texture, Some(&D3D11_RENDER_TARGET_VIEW_DESC {
				Format: format,
				ViewDimension: D3D11_RTV_DIMENSION_TEXTURE2D,
				Anonymous: D3D11_RENDER_TARGET_VIEW_DESC_0 {
					Texture2D: D3D11_TEX2D_RTV {
						MipSlice: 0,
					},
				},
			}), Some(&mut view_rt)).unwrap()};
		}
		
		let mut view_depth = None;
		if usage.contains(super::TextureUsage::DEPTH_STENCIL) {
			unsafe{self.device.CreateDepthStencilView(&texture, Some(&D3D11_DEPTH_STENCIL_VIEW_DESC {
				Format: format,
				ViewDimension: D3D11_DSV_DIMENSION_TEXTURE2D,
				Flags: 0,
				Anonymous: D3D11_DEPTH_STENCIL_VIEW_DESC_0 {
					Texture2D: D3D11_TEX2D_DSV {
						MipSlice: 0,
					},
				}
			}), Some(&mut view_depth)).unwrap()};
		}
		
		Rc::new(D3d11Texture {
			texture,
			context: self.context.clone(),
			view,
			view_rt,
			view_depth,
			width,
			height,
		})
	}
	
	fn create_buffer(&self, size: usize, usage: super::BufferUsage) -> super::Buffer {
		let mut buffer = None;
		unsafe{self.device.CreateBuffer(&D3D11_BUFFER_DESC {
			ByteWidth: size as u32,
			Usage: D3D11_USAGE_DYNAMIC,
			BindFlags:
				if usage.contains(super::BufferUsage::UNIFORM) {D3D11_BIND_CONSTANT_BUFFER.0 as u32} else {0} |
				if usage.contains(super::BufferUsage::INDEX) {D3D11_BIND_INDEX_BUFFER.0 as u32} else {0} |
				if usage.contains(super::BufferUsage::VERTEX) {D3D11_BIND_VERTEX_BUFFER.0 as u32} else {0},
			CPUAccessFlags:
				if usage.contains(super::BufferUsage::COPY_DST) {D3D11_CPU_ACCESS_WRITE.0 as u32} else {0} |
				if usage.contains(super::BufferUsage::COPY_SRC) {D3D11_CPU_ACCESS_READ.0 as u32} else {0},
			..Default::default()
		}, None, Some(&mut buffer)).unwrap()};
		
		Rc::new(D3d11Buffer {
			buffer: buffer.unwrap(),
			context: self.context.clone(),
			size,
		})
	}
	
	fn create_sampler(&self, address_u: super::SamplerAddress, address_v: super::SamplerAddress, min: super::SamplerFilter, mag: super::SamplerFilter) -> super::Sampler {
		let mut sampler = None;
		unsafe{self.device.CreateSamplerState(&D3D11_SAMPLER_DESC {
			Filter: match (min, mag) {
				(super::SamplerFilter::Nearest, super::SamplerFilter::Nearest) => D3D11_FILTER_MIN_MAG_MIP_POINT,
				(super::SamplerFilter::Nearest, super::SamplerFilter::Linear)  => D3D11_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT,
				(super::SamplerFilter::Linear,  super::SamplerFilter::Nearest) => D3D11_FILTER_MIN_LINEAR_MAG_MIP_POINT,
				(super::SamplerFilter::Linear,  super::SamplerFilter::Linear)  => D3D11_FILTER_MIN_MAG_LINEAR_MIP_POINT,
			},
			AddressU: match address_u {
				super::SamplerAddress::ClampToEdge => D3D11_TEXTURE_ADDRESS_CLAMP,
				super::SamplerAddress::Repeat => D3D11_TEXTURE_ADDRESS_WRAP,
				super::SamplerAddress::MirrorRepeat => D3D11_TEXTURE_ADDRESS_MIRROR,
				super::SamplerAddress::ClampToBorder => D3D11_TEXTURE_ADDRESS_BORDER,
			},
			AddressV: match address_v {
				super::SamplerAddress::ClampToEdge => D3D11_TEXTURE_ADDRESS_CLAMP,
				super::SamplerAddress::Repeat => D3D11_TEXTURE_ADDRESS_WRAP,
				super::SamplerAddress::MirrorRepeat => D3D11_TEXTURE_ADDRESS_MIRROR,
				super::SamplerAddress::ClampToBorder => D3D11_TEXTURE_ADDRESS_BORDER,
			},
			AddressW: D3D11_TEXTURE_ADDRESS_WRAP,
			..Default::default()
		}, Some(&mut sampler)).unwrap()};
		
		Rc::new(D3d11Sampler {
			sampler: sampler.unwrap(),
		})
	}
	
	fn render(&self,
		clear_color: &Option<[f32; 4]>,
		render_target: &super::Texture,
		depth_buffer: &super::Texture,
		materials: &HashMap<&'static str, super::Material>,
		objects: &crate::ObjectBuffer,
		camera: &crate::scene::Camera) {
		let render_target = render_target.as_any().downcast_ref::<D3d11Texture>().unwrap();
		let depth_buffer = depth_buffer.as_any().downcast_ref::<D3d11Texture>().unwrap();
		
		unsafe {
			if let Some(clear_color) = clear_color {
				self.context.ClearRenderTargetView(render_target.view_rt.as_ref().unwrap(), clear_color);
				self.context.ClearDepthStencilView(depth_buffer.view_depth.as_ref().unwrap(), D3D11_CLEAR_DEPTH.0, 1.0, 0);
			}
			
			self.context.RSSetViewports(Some(&[D3D11_VIEWPORT {
				TopLeftX: 0.0,
				TopLeftY: 0.0,
				Width: render_target.width as f32,
				Height: render_target.height as f32,
				MinDepth: 0.0,
				MaxDepth: 1.0,
			}]));
			
			self.context.OMSetRenderTargets(Some(&[Some(render_target.view_rt.as_ref().unwrap().clone())]), depth_buffer.view_depth.as_ref().unwrap());
			self.context.OMSetDepthStencilState(&self.depth_stencil_state, 0);
			self.context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
			self.context.RSSetState(&self.rasterizer_state);
			
			for obj in objects {
				let Some(obj) = obj else {continue};
				if !obj.get_visible() {continue};
				let material = materials.get(obj.get_material_id()).unwrap().as_any().downcast_ref::<D3d11Material>().unwrap();
				
				let mut uniform_buffer = None;
				self.device.CreateBuffer(&D3D11_BUFFER_DESC {
					ByteWidth: size_of::<super::Uniform>() as u32,
					Usage: D3D11_USAGE_DYNAMIC,
					BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
					CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
					..Default::default()
				}, None, Some(&mut uniform_buffer)).unwrap();
				let uniform_buffer = uniform_buffer.unwrap();
				
				let proj = camera.proj_matrix(render_target.width as f32 / render_target.height as f32);
				let uniform_data = super::Uniform {
					camera_view: camera.view,
					camera_view_inv: camera.view.inverse(),
					camera_proj: proj,
					camera_proj_inv: proj.inverse(),
					object: *obj.get_matrix(),
				};
				
				let mut data_map = D3D11_MAPPED_SUBRESOURCE::default();
				self.context.Map(&uniform_buffer, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut data_map)).unwrap();
				core::ptr::copy_nonoverlapping(&uniform_data, data_map.pData as _, 1);
				
				self.context.VSSetShader(Some(&material.vs), None);
				self.context.PSSetShader(Some(&material.ps), None);
				self.context.VSSetConstantBuffers(0, Some(&[Some(uniform_buffer.clone())]));
				self.context.PSSetConstantBuffers(0, Some(&[Some(uniform_buffer)]));
				self.context.IASetInputLayout(Some(&material.layout));
				
				for (i, (resource, bind)) in obj.get_shader_resources().iter().zip(&material.binds).enumerate() {
					let index = i as u32 + 1;
					
					match resource {
						super::ShaderResource::Buffer(buffer) => {
							let buffer = buffer.as_any().downcast_ref::<D3d11Buffer>().unwrap();
							
							if bind.stage.contains(super::MaterialBindStage::VERTEX) {
								self.context.VSSetConstantBuffers(index, Some(&[Some(buffer.buffer.clone())]));
							}
							
							if bind.stage.contains(super::MaterialBindStage::FRAGMENT) {
								self.context.PSSetConstantBuffers(index, Some(&[Some(buffer.buffer.clone())]));
							}
						}
						
						super::ShaderResource::Texture(texture) => {
							let texture = texture.as_any().downcast_ref::<D3d11Texture>().unwrap();
							
							if bind.stage.contains(super::MaterialBindStage::VERTEX) {
								self.context.VSSetShaderResources(index, Some(&[texture.view.clone()]));
							}
							
							if bind.stage.contains(super::MaterialBindStage::FRAGMENT) {
								self.context.PSSetShaderResources(index, Some(&[texture.view.clone()]));
							}
						}
						
						super::ShaderResource::Sampler(sampler) => {
							let sampler = sampler.as_any().downcast_ref::<D3d11Sampler>().unwrap();
							
							if bind.stage.contains(super::MaterialBindStage::VERTEX) {
								self.context.PSSetSamplers(index, Some(&[Some(sampler.sampler.clone())]));
							}
							
							if bind.stage.contains(super::MaterialBindStage::FRAGMENT) {
								self.context.PSSetSamplers(index, Some(&[Some(sampler.sampler.clone())]));
							}
						}
					}
				}
				
				self.context.IASetVertexBuffers(0, 1, Some(&Some(obj.get_vertex_buffer().as_any().downcast_ref::<D3d11Buffer>().unwrap().buffer.clone())), Some(&(size_of::<crate::Vertex>() as u32)), Some(&0));
				self.context.IASetIndexBuffer(&obj.get_index_buffer().as_any().downcast_ref::<D3d11Buffer>().unwrap().buffer, DXGI_FORMAT_R16_UINT, 0);
				self.context.DrawIndexed(obj.get_index_count(), 0, 0);
			}
			
			let mut cmdlist = None;
			self.context.FinishCommandList(true, Some(&mut cmdlist)).unwrap();
			let cmdlist = cmdlist.unwrap();
			self.device.GetImmediateContext().unwrap().ExecuteCommandList(Some(&cmdlist), true);
			self.context.ClearState();
		}
	}
	
	fn register_texture(&self, texture: &super::Texture) -> u64 {
		(self.texture_register)(texture)
	}
	
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
	
	fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
		self
	}
}

// ----------

pub struct D3d11Material {
	binds: Vec<super::MaterialBind>,
	vs: ID3D11VertexShader,
	ps: ID3D11PixelShader,
	layout: ID3D11InputLayout,
}

impl super::MaterialInner for D3d11Material {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn compile_shader(source: &str, entry: &str, target: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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

// ----------

pub struct D3d11Texture {
	texture: ID3D11Texture2D,
	context: Rc<ID3D11DeviceContext>,
	view: Option<ID3D11ShaderResourceView>,
	view_rt: Option<ID3D11RenderTargetView>,
	view_depth: Option<ID3D11DepthStencilView>,
	width: u32,
	height: u32,
}

impl D3d11Texture {
	pub fn get_view_ptr(&self) -> usize {
		self.view.as_ref().unwrap().as_raw() as _
	}
}

impl super::TextureInner for D3d11Texture {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
	
	fn set_data(&self, data: &[u8]) {
		let mut data_map = D3D11_MAPPED_SUBRESOURCE::default();
		unsafe{self.context.Map(&self.texture, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut data_map)).unwrap()};
		unsafe{core::ptr::copy_nonoverlapping(data.as_ptr(), data_map.pData as _, data.len())};
	}
}

// ----------

pub struct D3d11Buffer {
	buffer: ID3D11Buffer,
	context: Rc<ID3D11DeviceContext>,
	size: usize,
}

impl super::BufferInner for D3d11Buffer {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
	
	fn set_data(&self, data: &[u8]) {
		let mut data_map = D3D11_MAPPED_SUBRESOURCE::default();
		unsafe{self.context.Map(&self.buffer, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut data_map)).unwrap()};
		unsafe{core::ptr::copy_nonoverlapping(data.as_ptr(), data_map.pData as _, data.len())};
	}
	
	fn size(&self) -> usize {
		self.size
	}
}

// ----------

pub struct D3d11Sampler {
	sampler: ID3D11SamplerState,
}

impl super::SamplerInner for D3d11Sampler {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
}