# Changelog

All notable changes to this use case are documented in this file.
## [0.3.3](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-flightcrew@0.3.2...solverforge-flightcrew@0.3.3) (2026-11-10)


### Bug Fixes

* **runtime:** target SolverForge 0.25.1 across the bundle ([093fc61](https://github.com/SolverForge/solverforge-usecases/commit/093fc61c49983df6d6f0c0b3ba68408439a77869))


## [0.3.2](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-flightcrew@0.3.1...solverforge-flightcrew@0.3.2) (2026-11-09)


### Bug Fixes

* **runtime:** target SolverForge 0.25.0 across the bundle ([0ffbf64](https://github.com/SolverForge/solverforge-usecases/commit/0ffbf64b997b7c80279202f5c1690c62b463146d))


## [0.3.1](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-flightcrew@0.3.0...solverforge-flightcrew@0.3.1) (2026-11-03)


### Bug Fixes

* **routing:** identify the apps to Overpass with solverforge-maps 2.1.5 ([61d95ec](https://github.com/SolverForge/solverforge-usecases/commit/61d95ecabd67e1faa93dcaf209f0a7b0c01cf160))


## [0.3.0](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-flightcrew@0.2.0...solverforge-flightcrew@0.3.0) (2026-11-03)


### ⚠ BREAKING CHANGES

* **analysis:** analysis constraints report a numeric `weightMultiplier` instead of the `weight` string, and flightcrew, furnace and orders selector telemetry no longer carries `conflictRepair*` counters.

### Features

* **analysis:** report the constraint weightMultiplier on SolverForge 0.24.0 ([e86f8ce](https://github.com/SolverForge/solverforge-usecases/commit/e86f8ce293997118faf42cc173b757f4b614bc1f))


### Bug Fixes

* **build:** make every app's browser tests run from a standalone clone ([2dab038](https://github.com/SolverForge/solverforge-usecases/commit/2dab038c4de559a6e80b0134fd16e86ff170f9dc))
* **build:** run slow acceptance solves in the release profile ([6c279ef](https://github.com/SolverForge/solverforge-usecases/commit/6c279eff404d49015f1f8dca6349492b54de9ea2))
* **runtime:** target SolverForge 0.24.0 across the bundle ([32c9e75](https://github.com/SolverForge/solverforge-usecases/commit/32c9e75a3462a07c0d4e87f5e677c76c5dd8adda))


## [0.2.0](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-flightcrew@0.1.5...solverforge-flightcrew@0.2.0) (2026-10-27)


### ⚠ BREAKING CHANGES

* **telemetry:** telemetry responses replace the `phase` object with a `phases` array.

### Features

* **telemetry:** report per-phase solve telemetry on SolverForge 0.23.0 ([372c300](https://github.com/SolverForge/solverforge-usecases/commit/372c30052272948daa5045bbefcb2c394352cf1f))


### Bug Fixes

* **e2e:** honor Cargo target directory in browser servers ([556cc7a](https://github.com/SolverForge/solverforge-usecases/commit/556cc7aedeff3eaaa821917f72f4f596c98eb52a))


## [0.1.5](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-flightcrew@solverforge-flightcrew@0.1.4...solverforge-flightcrew@solverforge-flightcrew@0.1.5) (2026-10-18)


### Bug Fixes

* **runtime:** target SolverForge 0.22.0 across the bundle ([36303f3](https://github.com/SolverForge/solverforge-usecases/commit/36303f3f059b78e5618f61ebb7c920c80c5c115f))


## [0.1.4](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-flightcrew@solverforge-flightcrew@0.1.3...solverforge-flightcrew@solverforge-flightcrew@0.1.4) (2026-10-15)


### Bug Fixes

* **runtime:** target SolverForge 0.21.3 across the bundle ([9a87be4](https://github.com/SolverForge/solverforge-usecases/commit/9a87be40f5797a8c95d8b33b6af392497457a99a))


## [0.1.3](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-flightcrew@solverforge-flightcrew@0.1.2...solverforge-flightcrew@solverforge-flightcrew@0.1.3) (2026-10-14)


### Bug Fixes

* **runtime:** target SolverForge 0.21.0 across the bundle ([0c9732b](https://github.com/SolverForge/solverforge-usecases/commit/0c9732bb9a4546adbd1859f2974c16ffe0551b41))
* **runtime:** target SolverForge 0.21.2 across the bundle ([7a614c3](https://github.com/SolverForge/solverforge-usecases/commit/7a614c34a6614c1d649edee6426cfd459400e03d))


## 0.1.2 (2026-09-27)


### Bug Fixes

* **flightcrew:** target SolverForge 0.19.7 bbb8b77

## 0.1.1 (2026-09-25)


### Bug Fixes

* **ui:** show hard/soft type in flight crew score analysis f4d7956

## 0.1.0 (2026-09-21)


### Features

* **flightcrew:** port crew scheduling to SolverForge 36e2a7a
