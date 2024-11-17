struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct TransformUniform {
    model: mat4x4<f32>,
};

// Add light parameters
struct LightUniform {
    position: vec3<f32>,
    color: vec3<f32>,
    ambient: f32,
    diffuse: f32,
    specular: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> transform: TransformUniform;

@group(2) @binding(0)
var<uniform> light: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) world_normal: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    
    let world_position = transform.model * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    
    // Transform normal to world space
    let normal_matrix = mat3x3<f32>(
        transform.model[0].xyz,
        transform.model[1].xyz,
        transform.model[2].xyz
    );
    out.world_normal = normalize(normal_matrix * model.normal);
    
    out.clip_position = camera.view_proj * world_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let light_dir = normalize(light.position - in.world_position);
    
    // Ambient
    let ambient = light.color * light.ambient;
    
    // Diffuse
    let diff = max(dot(normal, light_dir), 0.0);
    let diffuse = light.color * (diff * light.diffuse);
    
    // Simple specular (view direction is approximated)
    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0) - in.world_position);
    let reflect_dir = reflect(-light_dir, normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let specular = light.color * (spec * light.specular);
    
    // Combine lighting
    let result = (ambient + diffuse + specular) * in.color;
    
    return vec4<f32>(result, 1.0);
}
