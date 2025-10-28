# bevy-mpm

[![license](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](#license)
[![docs](https://img.shields.io/badge/docs-WIP-orange.svg)](#documentation)
[![bevy](https://img.shields.io/badge/bevy-0.17-5b5d8a.svg)](https://bevyengine.org/)

**bevy-mpm** is an MLS-MPM sandbox for the [Bevy game engine](https://bevyengine.org/). It aims to power emergent worlds where sand, snow, water, and soft bodies interact within a unified solver.

---

## Design Goals

- **Native Bevy integration.** The solver runs inside the ECS—no external engines or FFI bindings.
- **Data-first materials.** Behaviour is described by parameter packs, so new matter types can be introduced without touching core code.
- **Phase-aware grid.** Grid nodes hold per-material accumulators, paving the way for phase separation instead of a single shared velocity field.
- **Focused scope.** We prioritise believable simulation features (APIC transfers, pressure controls, fracture hooks) over exhaustive checklists.

## Current Status

- MLS-MPM pipeline (particle bins, APIC, four-colour sweeps)
- Fluid parameter packs (`FluidParams`, `MaterialType::fluid`)
- Grid groundwork for per-material accumulators (`MaterialSlot` on `GridNode`)
- Phase mixing logic (friction/multi-material velocity solve) — in progress
- Granular & solid parameter packs with constitutive wrappers — planned
- Example gallery and documentation refresh — planned

## Getting Started

Add the crate to your project (WIP, crate not yet published on crates.io):

```toml
[dependencies]
bevy-mpm = { git = "https://github.com/mathi0/bevy-mpm" }
```

Enable the plugin in your Bevy app:

```rust
use bevy::prelude::*;
use bevy_mpm::{MpmPlugin, FluidParams, MaterialType};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MpmPlugin::default()))
        .add_systems(Startup, setup_particles)
        .run();
}

fn setup_particles(mut state: ResMut<bevy_mpm::MpmState>) {
    let water = MaterialType::fluid(FluidParams::water());
    let mut particle = bevy_mpm::Particle::zeroed(water);
    particle.position = Vec2::new(64.0, 72.0);
    particle.velocity = Vec2::new(0.0, -10.0);
    state.add_particle(particle);
}
```

## Example

The crate ships with a `basic_mpm` example showcasing the water preset, cursor-driven forces, and HUD diagnostics:

```sh
cargo run --example basic_mpm --release
```


## Documentation

Documentation is a work-in-progress. Stay tuned for migration guides and deep dives into material authoring and phase-aware simulation.

## Roadmap

- Granular & elastic parameter packs with constitutive wrappers
- Per-material mixing pass (friction/contact) to avoid particle smearing
- Scene assets and benchmark suite
- Documentation and tutorial series

## Contributing

Pull requests and issues are welcome! If you encounter a bug or want to propose a feature, open an issue so we can discuss scope.

Join the conversation on the [Bevy Discord](https://discord.gg/bevy) in `#ecosystem-crates`.

## License

Dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE) or <http://opensource.org/licenses/MIT>)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.

