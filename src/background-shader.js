/**
 * DAWN Installer - Dynamic Wavy Silk Dawn Shader
 * - Fixed top-left NaN bug: replaced fractional pow with safe Gaussian math
 * - Soft, dimmed-down, eye-friendly palette
 * - Smooth Gaussian transitions with zero hard edges
 * - Continuous, soothing multi-harmonic wave undulations
 * - Subtle vignette framing
 */

export class BackgroundShader {
  constructor(canvas) {
    this.canvas = canvas;
    this.gl = canvas.getContext('webgl', {
      alpha: false,
      antialias: true,
      powerPreference: 'high-performance'
    }) || canvas.getContext('experimental-webgl');

    if (!this.gl) {
      this.canvas.parentElement.classList.add('shader-fallback');
      return;
    }

    this.startTime = performance.now();
    this.animationFrameId = null;

    this.init();
  }

  init() {
    const gl = this.gl;

    const vsSource = `
      attribute vec2 a_position;
      void main() {
        gl_Position = vec4(a_position, 0.0, 1.0);
      }
    `;

    const fsSource = `
      precision highp float;

      uniform vec2 u_resolution;
      uniform float u_time;

      // 2D Simplex Noise for natural fluid turbulence
      vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
      vec2 mod289(vec2 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
      vec3 permute(vec3 x) { return mod289(((x*34.0)+1.0)*x); }

      float snoise(vec2 v) {
        const vec4 C = vec4(0.211324865405187, 0.366025403784439, -0.577350269189626, 0.024390243902439);
        vec2 i  = floor(v + dot(v, C.yy) );
        vec2 x0 = v -   i + dot(i, C.xx);
        vec2 i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
        vec4 x12 = x0.xyxy + C.xxzz;
        x12.xy -= i1;
        i = mod289(i);
        vec3 p = permute( permute( i.y + vec3(0.0, i1.y, 1.0 )) + i.x + vec3(0.0, i1.x, 1.0 ));
        vec3 m = max(0.5 - vec3(dot(x0,x0), dot(x12.xy,x12.xy), dot(x12.zw,x12.zw)), 0.0);
        m = m*m ;
        m = m*m ;
        vec3 x = 2.0 * fract(p * C.www) - 1.0;
        vec3 h = abs(x) - 0.5;
        vec3 ox = floor(x + 0.5);
        vec3 a0 = x - ox;
        m *= 1.79284291400159 - 0.85373472095314 * ( a0*a0 + h*h );
        vec3 g;
        g.x  = a0.x  * x0.x  + h.x  * x0.y;
        g.yz = a0.yz * x12.xz + h.yz * x12.yw;
        return 130.0 * dot(m, g);
      }

      void main() {
        vec2 uv = gl_FragCoord.xy / u_resolution.xy;
        float aspect = u_resolution.x / u_resolution.y;

        // Controlled speed for smooth, gentle wavy rhythm
        float t = u_time * 0.70;

        // ====================================================================
        // 1. Pronounced Multi-Scale Wavy Displacement Field
        // ====================================================================
        float waveY = sin(uv.x * 3.2 - t * 1.10) * 0.130 
                    + cos(uv.x * 5.0 + t * 0.85) * 0.065
                    + sin((uv.x + uv.y * 1.3) * 3.6 - t * 0.95) * 0.045;

        float waveX = cos(uv.y * 2.8 + t * 0.80) * 0.075
                    + sin((uv.y - uv.x * 1.1) * 3.2 + t * 0.60) * 0.040;

        float nWarp = snoise(vec2(uv.x * 2.0 - t * 0.35, uv.y * 2.0 + t * 0.28)) * 0.08;

        vec2 p = uv + vec2(waveX, waveY) + vec2(nWarp * 0.45);

        // ====================================================================
        // 2. Soft, Dimmed-Down Palette
        // ====================================================================
        vec3 cBottomDark    = vec3(0.020, 0.045, 0.130); // Soft deep midnight navy
        vec3 cMutedRoyal    = vec3(0.040, 0.130, 0.350); // Muted royal blue
        vec3 cTwilightMauve = vec3(0.190, 0.100, 0.290); // Soft dusty twilight purple
        vec3 cMorningRose   = vec3(0.520, 0.240, 0.440); // Gentle morning rose
        vec3 cWarmPeach     = vec3(0.700, 0.420, 0.280); // Dimmed warm peach
        vec3 cSunriseAmber  = vec3(0.820, 0.580, 0.350); // Soft dawn amber

        // ====================================================================
        // 3. Wavy Diagonal Dawn Progression Axis
        // ====================================================================
        float dynamicWaveShift = sin(p.x * 3.6 - t * 1.0) * 0.11 + cos(p.y * 3.0 + t * 0.75) * 0.07;
        float dawnCoord = p.y * 0.65 + (1.0 - p.x) * 0.45 + dynamicWaveShift;

        vec3 color = cBottomDark;
        float b1 = smoothstep(0.10, 0.55, dawnCoord);
        color = mix(color, cMutedRoyal, b1);

        float b2 = smoothstep(0.28, 0.72, dawnCoord);
        color = mix(color, cTwilightMauve, b2 * 0.75);

        float b3 = smoothstep(0.48, 0.90, dawnCoord);
        color = mix(color, cMorningRose, b3 * 0.65);

        // ====================================================================
        // 4. Undulating Silk Ribbon Waves (Safe Gaussian math)
        // ====================================================================
        float bluePath = 0.50 
                       + sin(p.x * 2.8 - t * 0.95) * 0.16 
                       + cos(p.x * 4.6 + t * 0.70) * 0.07;
        float distBlue = (p.y - bluePath) / 0.21;
        float wBlue = exp(-distBlue * distBlue);
        color = mix(color, cMutedRoyal * 1.30, wBlue * 0.48);

        float rosePath = 0.62 
                       + cos(p.x * 3.0 - t * 0.85 + 1.2) * 0.16 
                       + sin(p.x * 2.2 + t * 0.55) * 0.07;
        float distRose = (p.y - rosePath) / 0.21;
        float wRose = exp(-distRose * distRose);
        color = mix(color, cMorningRose * 1.10, wRose * 0.52);

        float purplePath = 0.28 
                         + sin(p.x * 3.2 - t * 0.75 + 1.8) * 0.13 
                         + cos(p.x * 4.8 + t * 0.50) * 0.06;
        float distPurple = (p.y - purplePath) / 0.19;
        float wPurple = exp(-distPurple * distPurple);
        color = mix(color, cTwilightMauve * 0.85, wPurple * 0.42);

        // ====================================================================
        // 5. Soft Top-Left Sunrise Glow (BUGFIX: Safe Gaussian, No negative pow)
        // ====================================================================
        vec2 sunAnchor = vec2(0.04, 0.96);
        float distSun = length((uv - sunAnchor) * vec2(1.2, 1.1));
        float sunRadius = 0.44 + sin(t * 0.8) * 0.03;
        float normSun = distSun / sunRadius;
        float sunMask = exp(-normSun * normSun);

        vec3 sunriseGlow = mix(cWarmPeach, cSunriseAmber, smoothstep(0.20, 0.70, sunMask));
        color = mix(color, sunriseGlow, clamp(sunMask * 0.85, 0.0, 1.0));

        // ====================================================================
        // 6. Deep Dark Bottom Anchor
        // ====================================================================
        float bottomShadow = smoothstep(0.38, 0.0, uv.y);
        color = mix(color, cBottomDark, bottomShadow * 0.65);

        // ====================================================================
        // 7. Subtle Cinematic Vignette
        // ====================================================================
        vec2 vigCoord = (uv - 0.5) * vec2(aspect * 0.85, 1.0);
        float vigDist = length(vigCoord);
        float vignette = smoothstep(0.90, 0.25, vigDist);
        vignette = mix(0.80, 1.0, vignette);

        color = mix(cBottomDark * 0.9, color, vignette);

        // Anti-banding fine dither
        float dither = fract(sin(dot(gl_FragCoord.xy, vec2(12.9898, 78.233))) * 43758.5453) * 0.006;
        color += dither;

        gl_FragColor = vec4(clamp(color, 0.0, 1.0), 1.0);
      }
    `;

    const compileShader = (src, type) => {
      const shader = gl.createShader(type);
      gl.shaderSource(shader, src);
      gl.compileShader(shader);
      if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        console.error('Shader compile error:', gl.getShaderInfoLog(shader));
        gl.deleteShader(shader);
        return null;
      }
      return shader;
    };

    const vs = compileShader(vsSource, gl.VERTEX_SHADER);
    const fs = compileShader(fsSource, gl.FRAGMENT_SHADER);
    if (!vs || !fs) return;

    this.program = gl.createProgram();
    gl.attachShader(this.program, vs);
    gl.attachShader(this.program, fs);
    gl.linkProgram(this.program);

    if (!gl.getProgramParameter(this.program, gl.LINK_STATUS)) {
      console.error('Program link error:', gl.getProgramInfoLog(this.program));
      return;
    }

    gl.useProgram(this.program);

    const positions = new Float32Array([
      -1.0, -1.0,
       1.0, -1.0,
      -1.0,  1.0,
      -1.0,  1.0,
       1.0, -1.0,
       1.0,  1.0,
    ]);

    const posBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, posBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);

    const aPos = gl.getAttribLocation(this.program, 'a_position');
    gl.enableVertexAttribArray(aPos);
    gl.vertexAttribPointer(aPos, 2, gl.FLOAT, false, 0, 0);

    this.uResLoc = gl.getUniformLocation(this.program, 'u_resolution');
    this.uTimeLoc = gl.getUniformLocation(this.program, 'u_time');

    this.handleResize = this.resize.bind(this);
    window.addEventListener('resize', this.handleResize);
    this.resize();

    this.render();
  }

  resize() {
    const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
    const width = this.canvas.clientWidth || window.innerWidth;
    const height = this.canvas.clientHeight || window.innerHeight;

    if (this.canvas.width !== width * dpr || this.canvas.height !== height * dpr) {
      this.canvas.width = Math.floor(width * dpr);
      this.canvas.height = Math.floor(height * dpr);
    }

    if (this.gl) {
      this.gl.viewport(0, 0, this.canvas.width, this.canvas.height);
    }
  }

  render() {
    if (!this.gl || !this.program) return;

    const gl = this.gl;
    gl.useProgram(this.program);

    const currentTime = (performance.now() - this.startTime) * 0.001;

    gl.uniform2f(this.uResLoc, this.canvas.width, this.canvas.height);
    gl.uniform1f(this.uTimeLoc, currentTime);

    gl.drawArrays(gl.TRIANGLES, 0, 6);

    this.animationFrameId = requestAnimationFrame(this.render.bind(this));
  }

  destroy() {
    if (this.animationFrameId) cancelAnimationFrame(this.animationFrameId);
    window.removeEventListener('resize', this.handleResize);
  }
}
