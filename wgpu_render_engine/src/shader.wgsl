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
    light_space_matrix: mat4x4<f32>, 
}
struct VertexInput { 
    @location(0) position: vec3<f32>, 
    @location(1) color: vec3<f32>, 
    @location(2) normal: vec3<f32>, 
}
struct VertexOutput { 
    @builtin(position) clip_position: vec4<f32>, 
    @location(0) world_position: vec3<f32>, 
    @location(1) world_normal: vec3<f32>, 
    @location(2) color: vec3<f32>, 
}
@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> transform: TransformUniform;
@group(2) @binding(0) var<uniform> light: LightUniform;

fn simple_ground_texture(pos: vec3<f32>) -> vec3<f32> {
    // Create a grid-like pattern for the ground
    let grid_scale = 2.0;
    let x = abs(fract(pos.x * grid_scale) - 0.5);
    let z = abs(fract(pos.z * grid_scale) - 0.5);
    
    // Base ground color
    let base_green = vec3<f32>(0.2, 0.5, 0.2);
    let dark_green = vec3<f32>(0.15, 0.4, 0.15);
    
    // Create a subtle grid effect
    let grid_intensity = smoothstep(0.45, 0.5, max(x, z));
    return mix(base_green, dark_green, grid_intensity);
}

@vertex fn vs_main(model: VertexInput) -> VertexOutput { 
    var out: VertexOutput; 
    
    var world_position = transform.model * vec4<f32>(model.position, 1.0);
    
    out.world_position = world_position.xyz;
    
    let normal_matrix = mat3x3<f32>(
        transform.model[0].xyz,
        transform.model[1].xyz,
        transform.model[2].xyz
    );
    out.world_normal = normalize(normal_matrix * model.normal);
    
    out.clip_position = camera.view_proj * vec4<f32>(out.world_position, 1.0); 
    out.color = model.color;
    
    return out; 
} 

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let light_dir = normalize(light.position - in.world_position);
    let view_dir = normalize(camera.view_position - in.world_position);
    
    // Determine if this is a ground vertex based on vertex color
    let is_ground = in.color.g > 0.4 && in.color.r < 0.3;  // Ground vertices have more green
    
    // Choose base color based on whether it's ground or pyramid
    let base_color = select(
        in.color,  // Use vertex color for pyramid
        simple_ground_texture(in.world_position),  // Use ground texture for ground
        is_ground
    );
    
    // Ambient term
    let ambient = light.color * light.ambient;
    
    // Diffuse term with enhanced visibility
    let diff = max(dot(normal, light_dir), 0.3);
    let diffuse = light.color * diff * light.diffuse;
    
    // Specular term (reduced for ground)
    let halfway_dir = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, halfway_dir), 0.0), 32.0);
    let specular = light.color * spec * light.specular * select(1.0, 0.2, is_ground);
    
    // Combine lighting terms
    let final_color = base_color * (ambient + diffuse) + specular;
    
    return vec4<f32>(final_color, 1.0);
}

