// HACK: Somewhat cheating by drawing a triangle that's sufficiently bigger than the
// screenspace and still mapping the fragment shader correctly after scaling and clipping

const vec2 verts[3] = vec2[3](
    vec2(-1.0f, 1.0f),  // bottom left
    vec2(3.0f, 1.0f),   // twice as far as bottom right (from bottom left)
    vec2(-1.0f, -3.0f)  // twice as far as top left (from bottom left)
);

out vec2 uv;

void main() {
    vec2 vert = verts[gl_VertexID];
    uv = (vert+1.0)/2.0; // creates [0,1]x[0,1] range in the visible portion (whole tex)
    gl_Position = vec4(vert, 0.0, 1.0);
}
