use nalgebra as na;
use sdl2::{keyboard::Scancode, mouse::MouseState, video::Window};
use wgpu::util::DeviceExt;

use crate::{
    camera::{Camera, CameraUniform},
    light::{DrawLight, LightUniform},
    model::{DrawModel, Model, ModelVertex, Vertex},
    pipeline::Pipeline,
    resources,
    texture::Texture,
};

struct Instance {
    position: na::Vector3<f32>,
    rotation: na::Quaternion<f32>,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        let unit_quaternion = na::UnitQuaternion::from_quaternion(self.rotation);
        let rotation_matrix = na::Matrix4::from(unit_quaternion.to_rotation_matrix());
        let translation_matrix = na::Matrix4::from(na::Translation3::from(self.position));
        let result_matrix = translation_matrix * rotation_matrix;
        InstanceRaw {
            model: result_matrix.into(),
            normal: na::Matrix3::from(unit_quaternion.to_rotation_matrix()).into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl Vertex for InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7=> Float32x4, 8=>Float32x4, 9=>Float32x3, 10=>Float32x3, 11=>Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATRIBUTES,
        }
    }
}

const NUM_INSTANCES_PER_ROW: u32 = 10;

pub struct Engine<'a> {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: (u32, u32),
    window: &'a Window,
    render_pipeline: wgpu::RenderPipeline,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: Texture,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    pub frame_number: usize,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    depth_texture: Texture,
    obj_model: Model,
    obj_2: Model,
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_pipeline: wgpu::RenderPipeline,
}

impl<'a> Engine<'a> {
    pub async fn new(window: &'a Window) -> Engine<'a> {
        let size = window.size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0,
            height: size.1,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let diffuse_bytes = include_bytes!(".././images/dude.png");
        let diffuse_texture =
            Texture::from_bytes(&device, &queue, diffuse_bytes, "dude.png").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        let mut camera = Camera {
            position: na::Vector3::new(0.0, 0.0, -2.0),
            screen_up: na::Unit::new_normalize(na::Vector3::new(0.0, -1.0, 0.0)),
            screen_right: na::Unit::new_normalize(na::Vector3::new(-1.0, 0.0, 0.0)),
            view_matrix: na::Matrix4::identity(),
            rotation_matrix: na::Matrix3::identity(),
            pitch: 0.0,
            yaw: 0.0,
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            distance: 0.1,
            last_mouse_pos: None,
            sensitivity: 0.01,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&mut camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let epsilon = 1e-6;
        let space_between = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = space_between * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = space_between * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let position = na::Vector3::new(x as f32, 0.0, z as f32);

                    let rotation = if position.norm() < epsilon {
                        na::UnitQuaternion::from_axis_angle(
                            &na::Unit::new_normalize(na::Vector3::new(0.0, 0.0, 1.0)),
                            0.0,
                        )
                    } else {
                        na::UnitQuaternion::from_axis_angle(
                            &na::Unit::new_normalize(position),
                            45.0,
                        )
                    };
                    Instance {
                        position,
                        rotation: *rotation,
                    }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let obj_model =
            resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
                .await
                .unwrap();

        let obj_2 = resources::load_model("blade.obj", &device, &queue, &texture_bind_group_layout)
            .await
            .unwrap();

        let light_uniform = LightUniform::new([2.0, 2.0, 2.0], [1.0, 1.0, 1.0]);

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let shader = wgpu::include_wgsl!(".././shaders/shader.wgsl");

        let render_pipeline = Pipeline::new(
            &device,
            &render_pipeline_layout,
            config.format,
            Some(Texture::DEPTH_FORMAT),
            &[ModelVertex::desc(), InstanceRaw::desc()],
            shader,
        )
        .pipeline;

        let light_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light pipeline layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
        let shader = wgpu::include_wgsl!(".././shaders/light.wgsl");
        let light_pipeline = Pipeline::new(
            &device,
            &light_pipeline_layout,
            config.format,
            Some(Texture::DEPTH_FORMAT),
            &[ModelVertex::desc()],
            shader,
        )
        .pipeline;

        Engine {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            diffuse_bind_group,
            diffuse_texture,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            frame_number: 0,
            instances,
            instance_buffer,
            depth_texture,
            obj_model,
            obj_2,
            light_uniform,
            light_buffer,
            light_bind_group,
            light_pipeline,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.config.width = new_size.0;
            self.config.height = new_size.1;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    pub fn input(&mut self, scancodes: &Vec<Scancode>, mouse_state: &MouseState) {
        self.camera.transform_camera(scancodes, mouse_state);
    }

    pub fn update(&mut self) {
        self.camera_uniform.update_view_proj(&mut self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        let old_position = na::Vector3::from(self.light_uniform.position);
        self.light_uniform.position =
            (na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.1) * old_position)
                .into();
        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            render_pass.set_pipeline(&self.light_pipeline);
            render_pass.draw_light_model(
                &self.obj_model,
                &self.camera_bind_group,
                &self.light_bind_group,
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw_model_instanced(
                &self.obj_model,
                0..self.instances.len() as u32,
                &self.camera_bind_group,
                &self.light_bind_group,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
