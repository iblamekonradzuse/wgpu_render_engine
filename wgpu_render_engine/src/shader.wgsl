// Camera, Transform, and Light Uniforms
struct CameraUniform { 
    view_proj: mat4x4<f32>, 
    view_position: vec3<f32>, 
}

struct TransformUniform { 
    model: mat4x4<f32>, 
}

struct LightUniform { 
    position: vec3<f32>, 
    color: vec3<f32>, 
    ambient: f32, 
    diffuse: f32, 
    specular: f32, 
    light_space_matrix: mat4x4<f32>, // For shadow mapping
}

// Vertex input and output structures
struct VertexInput { 
    @location(0) position: vec3<f32>, 
    @location(1) color: vec3<f32>, 
    @location(2) normal: vec3<f32>, 
}

struct VertexOutput { 
    @builtin(position) clip_position: vec4<f32>, 
    @location(0) color: vec3<f32>, 
    @location(1) world_position: vec3<f32>, 
    @location(2) world_normal: vec3<f32>, 
    @location(3) light_space_position: vec4<f32>, // For shadow mapping
}

// Uniforms bindings
@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> transform: TransformUniform;
@group(2) @binding(0) var<uniform> light: LightUniform;

// Optional shadow map texture (commented out for now)
// @group(2) @binding(1) var shadow_sampler: sampler;
// @group(2) @binding(2) var shadow_texture: texture_depth_2d;

@vertex fn vs_main(model: VertexInput) -> VertexOutput { 
    var out: VertexOutput; 
    out.color = model.color;

    // World space transformations
    let world_position = transform.model * vec4<f32>(model.position, 1.0); 
    out.world_position = world_position.xyz;

    // Transform normal to world space 
    let normal_matrix = mat3x3<f32>( 
        transform.model[0].xyz, 
        transform.model[1].xyz, 
        transform.model[2].xyz 
    ); 
    out.world_normal = normalize(normal_matrix * model.normal);

    // Compute light space position for potential shadow mapping
    out.light_space_position = light.light_space_matrix * world_position;

    // Final clip space position
    out.clip_position = camera.view_proj * world_position; 
    
    return out; 
} 

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { 
    let normal = normalize(in.world_normal); 
    let light_dir = normalize(light.position - in.world_position);
    let view_dir = normalize(camera.view_position - in.world_position);

    // Ambient component
    let ambient = light.color * light.ambient;

    // Diffuse component
    let diff = max(dot(normal, light_dir), 0.0); 
    let diffuse = light.color * (diff * light.diffuse);

    // Specular component
    let reflect_dir = reflect(-light_dir, normal); 
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0); 
    let specular = light.color * (spec * light.specular);

    // Shadow calculation placeholder
    // var shadow_factor = 1.0;
    // TODO: Implement shadow mapping logic here

    // Combine lighting
    let result = (ambient + diffuse + specular) * in.color;
    
    return vec4<f32>(result, 1.0); 
}
