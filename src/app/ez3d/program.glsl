varying vec4 v_color;

#ifdef VERTEX_SHADER
attribute vec3 a_pos;
attribute vec4 a_color;
attribute vec3 a_normal;

attribute vec3 i_pos;
attribute float i_size;
attribute float i_rotation;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;

#define AMBIENT 0.3

void main()
{
    vec3 pos = a_pos;
    pos.xy = rotate(pos.xy, i_rotation);
    vec3 normal = a_normal;
    normal.xy = rotate(normal.xy, i_rotation);
    float light = AMBIENT + max(0.0, dot(a_normal, normalize(vec3(10.0, 3.0, 5.0)))) * (1.0 - AMBIENT);
    v_color = a_color;
    v_color.xyz *= light;
    gl_Position = u_projection_matrix * u_view_matrix * vec4(i_pos + pos * i_size, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main()
{
    gl_FragColor = v_color;
}
#endif