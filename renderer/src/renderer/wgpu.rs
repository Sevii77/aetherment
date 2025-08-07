use std::{collections::HashMap, rc::Rc};

pub struct WgpuRenderer {
	device: wgpu::Device,
	queue: wgpu::Queue,
	texture_register: Box<dyn Fn(&super::Texture) -> u64>,
	
	depth_stencil_state: wgpu::DepthStencilState,
	basic_vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,
}

impl WgpuRenderer {
	pub fn new(device: wgpu::Device, queue: wgpu::Queue, texture_register: Box<dyn Fn(&super::Texture) -> u64>) -> Self {
		let basic_vertex_buffer_layout = wgpu::VertexBufferLayout {
			array_stride: 64,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					format: wgpu::VertexFormat::Float32x2,
					offset: 0,
					shader_location: 0,
				},
				wgpu::VertexAttribute {
					format: wgpu::VertexFormat::Float32x3,
					offset: 8,
					shader_location: 1,
				},
				wgpu::VertexAttribute {
					format: wgpu::VertexFormat::Float32x3,
					offset: 20,
					shader_location: 2,
				},
				wgpu::VertexAttribute {
					format: wgpu::VertexFormat::Float32x4,
					offset: 32,
					shader_location: 3,
				},
				wgpu::VertexAttribute {
					format: wgpu::VertexFormat::Float32x4,
					offset: 48,
					shader_location: 4,
				},
			],
		};
		
		let depth_stencil_state = wgpu::DepthStencilState {
			format: wgpu::TextureFormat::Depth32Float,
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::Less,
			bias: Default::default(),
			stencil: Default::default(),
		};
		
		Self {
			device,
			queue,
			texture_register,
			
			depth_stencil_state,
			basic_vertex_buffer_layout,
		}
	}
}

impl super::RendererInner for WgpuRenderer {
	fn create_material(&self, shader: &str, binds: &[super::MaterialBind]) -> super::Material {
		let mut bind_group_layout_entries = vec![
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}
		];
		
		for (i, v) in binds.into_iter().enumerate() {
			bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
				binding: i as u32 + 1,
				visibility: wgpu::ShaderStages::from_bits_retain(v.stage.bits()),
				ty: match v.typ {
					super::MaterialBindType::Buffer =>
						wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
					super::MaterialBindType::Texture =>
						wgpu::BindingType::Texture {
							sample_type: wgpu::TextureSampleType::Float{filterable: true},
							view_dimension: wgpu::TextureViewDimension::D2,
							multisampled: false,
						},
					super::MaterialBindType::Sampler =>
						wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
				},
				count: None,
			});
		}
		
		let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: None,
			entries: &bind_group_layout_entries,
		});
		
		let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});
		
		let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(shader)),
		});
		
		let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: Some("vs_main"),
				compilation_options: Default::default(),
				buffers: &[self.basic_vertex_buffer_layout.clone()],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: Some("fs_main"),
				compilation_options: Default::default(),
				targets: &[Some(wgpu::ColorTargetState {
					format: wgpu::TextureFormat::Rgba8Unorm,
					blend: None,
					write_mask: wgpu::ColorWrites::ALL,
				})]
			}),
			primitive: wgpu::PrimitiveState {
				cull_mode: Some(wgpu::Face::Back),
				..Default::default()
			},
			depth_stencil: Some(self.depth_stencil_state.clone()),
			multisample: Default::default(),
			multiview: None,
			cache: None,
		});
		
		Rc::new(WgpuMaterial {
			pipeline,
		})
	}
	
	fn create_texture(&self, width: u32, height: u32, format: super::TextureFormat, usage: super::TextureUsage) -> super::Texture {
		let texture = self.device.create_texture(&wgpu::TextureDescriptor {
			label: None,
			size: wgpu::Extent3d {
				width: width,
				height: height,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: match format {
				super::TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
				super::TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
				super::TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
			},
			usage:
				if usage.contains(super::TextureUsage::COPY_SRC) {wgpu::TextureUsages::COPY_SRC} else {wgpu::TextureUsages::empty()} |
				if usage.contains(super::TextureUsage::COPY_DST) {wgpu::TextureUsages::COPY_DST} else {wgpu::TextureUsages::empty()} |
				if usage.contains(super::TextureUsage::TEXTURE_BINDING) {wgpu::TextureUsages::TEXTURE_BINDING} else {wgpu::TextureUsages::empty()} |
				if usage.contains(super::TextureUsage::RENDER_TARGET) {wgpu::TextureUsages::RENDER_ATTACHMENT} else {wgpu::TextureUsages::empty()} |
				if usage.contains(super::TextureUsage::DEPTH_STENCIL) {wgpu::TextureUsages::RENDER_ATTACHMENT} else {wgpu::TextureUsages::empty()},
			view_formats: &[],
		});
		
		let view = texture.create_view(&Default::default());
		
		Rc::new(WgpuTexture {
			queue: self.queue.clone(),
			texture: texture,
			view,
			bbp: format.bbp(),
		})
	}
	
	fn create_buffer(&self, size: usize, usage: super::BufferUsage) -> super::Buffer {
		let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			size: size as u64,
			usage: wgpu::BufferUsages::from_bits_retain(usage.bits()),
			mapped_at_creation: false,
		});
		
		Rc::new(WgpuBuffer {
			queue: self.queue.clone(),
			buffer,
		})
	}
	
	fn create_sampler(&self, address_u: super::SamplerAddress, address_v: super::SamplerAddress, min: super::SamplerFilter, mag: super::SamplerFilter) -> super::Sampler {
		let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
			label: None,
			address_mode_u: match address_u {
				super::SamplerAddress::ClampToEdge => wgpu::AddressMode::ClampToEdge,
				super::SamplerAddress::Repeat => wgpu::AddressMode::Repeat,
				super::SamplerAddress::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
				super::SamplerAddress::ClampToBorder => wgpu::AddressMode::ClampToBorder,
			},
			address_mode_v: match address_v {
				super::SamplerAddress::ClampToEdge => wgpu::AddressMode::ClampToEdge,
				super::SamplerAddress::Repeat => wgpu::AddressMode::Repeat,
				super::SamplerAddress::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
				super::SamplerAddress::ClampToBorder => wgpu::AddressMode::ClampToBorder,
			},
			mag_filter: match mag {
				super::SamplerFilter::Nearest => wgpu::FilterMode::Nearest,
				super::SamplerFilter::Linear => wgpu::FilterMode::Linear,
			},
			min_filter: match min {
				super::SamplerFilter::Nearest => wgpu::FilterMode::Nearest,
				super::SamplerFilter::Linear => wgpu::FilterMode::Linear,
			},
			..Default::default()
		});
		
		Rc::new(WgpuSampler {
			sampler,
		})
	}
	
	fn render(&self,
		clear_color: &Option<[f32; 4]>,
		render_target: &super::Texture,
		depth_buffer: &super::Texture,
		materials: &HashMap<&'static str, super::Material>,
		objects: &crate::ObjectBuffer,
		camera: &crate::scene::Camera) {
		let render_target = render_target.as_any().downcast_ref::<WgpuTexture>().unwrap();
		let depth_buffer = depth_buffer.as_any().downcast_ref::<WgpuTexture>().unwrap();
		
		let mut encoder = self.device.create_command_encoder(&Default::default());
		let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: None,
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &render_target.view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: clear_color.map_or(wgpu::LoadOp::Load, |v| wgpu::LoadOp::Clear(wgpu::Color{r: v[0] as f64, g: v[1] as f64, b: v[2] as f64, a: v[3] as f64})),
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
				view: &depth_buffer.view,
				depth_ops: Some(wgpu::Operations {
					load: wgpu::LoadOp::Clear(1.0),
					store: wgpu::StoreOp::Store,
				}),
				stencil_ops: None,
			}),
			timestamp_writes: None,
			occlusion_query_set: None,
		});
		
		for obj in objects {
			let Some(obj) = obj else {continue};
			if !obj.get_visible() {continue};
			let material = materials.get(obj.get_material_id()).unwrap().as_any().downcast_ref::<WgpuMaterial>().unwrap();
			
			let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
				label: None,
				size: size_of::<super::Uniform>() as u64,
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false,
			});
			
			let proj = camera.proj_matrix(render_target.texture.width() as f32 / render_target.texture.height() as f32);
			self.queue.write_buffer(&uniform_buffer, 0, &bytemuck::cast_slice(&[super::Uniform {
				camera_view: camera.view,
				camera_view_inv: camera.view.inverse(),
				camera_proj: proj,
				camera_proj_inv: proj.inverse(),
				object: *obj.get_matrix(),
			}]));
			
			let mut bind_group_entries = vec![
				wgpu::BindGroupEntry {
					binding: 0,
					resource: uniform_buffer.as_entire_binding(),
				}
			];
			
			for (i, v) in obj.get_shader_resources().into_iter().enumerate() {
				bind_group_entries.push(wgpu::BindGroupEntry {
					binding: i as u32 + 1,
					resource: match v {
						super::ShaderResource::Buffer(buffer) =>
							buffer.as_any().downcast_ref::<WgpuBuffer>().unwrap().buffer.as_entire_binding(),
						super::ShaderResource::Texture(texture) =>
							wgpu::BindingResource::TextureView(&texture.as_any().downcast_ref::<WgpuTexture>().unwrap().view),
						super::ShaderResource::Sampler(sampler) =>
							wgpu::BindingResource::Sampler(&sampler.as_any().downcast_ref::<WgpuSampler>().unwrap().sampler),
					}
				})
			}
			
			let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: None,
				layout: &material.pipeline.get_bind_group_layout(0),
				entries: &bind_group_entries,
			});
			
			rpass.set_pipeline(&material.pipeline);
			rpass.set_bind_group(0, &bind_group, &[]);
			rpass.set_vertex_buffer(0, obj.get_vertex_buffer().as_any().downcast_ref::<WgpuBuffer>().unwrap().buffer.slice(..));
			rpass.set_index_buffer(obj.get_index_buffer().as_any().downcast_ref::<WgpuBuffer>().unwrap().buffer.slice(..), wgpu::IndexFormat::Uint16);
			rpass.draw_indexed(0..obj.get_index_count(), 0, 0..1);
		}
		
		drop(rpass);
		self.queue.submit([encoder.finish()]);
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

pub struct WgpuMaterial {
	pipeline: wgpu::RenderPipeline,
}

impl super::MaterialInner for WgpuMaterial {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
}

// ----------

pub struct WgpuTexture {
	queue: wgpu::Queue,
	texture: wgpu::Texture,
	view: wgpu::TextureView,
	bbp: u32,
}

impl WgpuTexture {
	pub fn get_view(&self) -> &wgpu::TextureView {
		&self.view
	}
}

impl super::TextureInner for WgpuTexture {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
	
	fn set_data(&self, data: &[u8]) {
		self.queue.write_texture(
			self.texture.as_image_copy(),
			data,
			wgpu::TexelCopyBufferLayout {
				offset: 0,
				bytes_per_row: Some(self.texture.width() * self.bbp / 8),
				rows_per_image: None,
			},
			self.texture.size()
		);
	}
}

// ----------

pub struct WgpuBuffer {
	queue: wgpu::Queue,
	buffer: wgpu::Buffer,
}

impl super::BufferInner for WgpuBuffer {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
	
	fn set_data(&self, data: &[u8]) {
		self.queue.write_buffer(&self.buffer, 0, data);
	}
}

// ----------

pub struct WgpuSampler {
	sampler: wgpu::Sampler,
}

impl super::SamplerInner for WgpuSampler {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
}