precision mediump float; // might be 16-bit? don't set globally?
in vec2 uv;
out vec4 color;
uniform float t;

float mult(float x, float y) {
    return x*y;
}

float add(float x, float y) {
    return x+y;
}

void main() {
    float u = uv.x;
    float v = uv.y;
    float r = mult(add(sin(t),1.0),0.5);
    float g = v;
    float b = u;
    color = vec4(r, g, b, 1.0);
}
