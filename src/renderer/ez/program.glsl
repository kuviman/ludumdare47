varying vec4 v_color;
varying vec2 v_vt;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;

attribute vec2 i_pos;
attribute float i_rotation;
attribute vec2 i_size;
attribute vec4 i_color;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;

void main()
{
    v_vt = (a_pos + 1.0) / 2.0;
    v_color = i_color;
    float s = sin(i_rotation), c = cos(i_rotation);
    gl_Position = u_projection_matrix * u_view_matrix * vec4(i_pos + mat2(c, -s, s, c) * a_pos * i_size, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
void main()
{
    gl_FragColor = texture2D(u_texture, vec2(v_vt.x, 1.0 - v_vt.y)) * v_color;
}
#endif