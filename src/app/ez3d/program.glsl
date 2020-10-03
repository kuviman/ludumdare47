varying vec4 v_color;

#ifdef VERTEX_SHADER
attribute vec3 a_pos;
attribute vec4 a_color;

attribute vec3 i_pos;
attribute float i_size;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;

void main()
{
    v_color = a_color;
    gl_Position = u_projection_matrix * u_view_matrix * vec4(i_pos + a_pos * i_size, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main()
{
    gl_FragColor = v_color;
}
#endif