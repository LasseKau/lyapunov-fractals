Lyapunov Fractals
=====================

Lyapunov fractal visualization using the [miniquad](https://github.com/not-fl3/miniquad) library. Zoom in and out of the fractal by clicking on the screen, color palette and logistic map parameters can be randomly updated by pressing the spacebar.

Web Build
----------------------------

The latest web build is available [here](https://lyapunov-fractals.netlify.app/).

Building from source
----------------------------

Depedencies:

The main depdency: the rust compiler.   
To get it, follow [rustup.rs](https://rustup.rs/) instructions.

On web, windows and mac os no other external depdendecies are required.
On linux followed libs may be required: 
```
apt install libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
```

Inspiration
----------------------------
This project was made using the following resources:

- [miniquad-mandelbrot](https://github.com/PonasKovas/miniquad-mandelbrot/blob/master/src/main.rs)
- [Wikipedia: Lyapunov fractal](https://en.wikipedia.org/wiki/Lyapunov_fractal)
- [Lyapunov Fractal by Frankovich](https://frankovich.dev/blog/lyapunov-fractal)


