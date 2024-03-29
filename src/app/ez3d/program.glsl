varying vec4 v_color;

#ifdef VERTEX_SHADER
attribute vec3 a_pos;
attribute vec4 a_color;
attribute vec3 a_normal;
attribute float a_emission;

attribute vec3 i_pos;
attribute float i_size;
attribute float i_rotation;
attribute vec4 i_color;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;

uniform vec3 u_light_direction;

#define AMBIENT vec3(0.3, 0.3, 0.3)
#define AMBIENT2 vec3(0.2, 0.2, 0.2)

void main()
{
    vec3 pos = a_pos;
    pos.xy = rotate(pos.xy, -i_rotation);
    vec3 normal = a_normal;
    normal.xy = rotate(normal.xy, i_rotation);
    vec3 light = AMBIENT + max(0.0, dot(a_normal, vec3(0.0, 0.0, 1.0))) * AMBIENT2 + max(0.0, dot(a_normal, u_light_direction)) * (vec3(1.0) - AMBIENT - AMBIENT2);
    vec4 color = a_color * i_color;
    v_color = vec4(color.xyz * min(light + vec3(a_emission), vec3(1.0)), color.w);
    gl_Position = u_projection_matrix * u_view_matrix * vec4(i_pos + pos * i_size, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main()
{
    gl_FragColor = v_color;
}
#endif