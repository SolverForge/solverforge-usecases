# Changelog

All notable changes to this use case are documented in this file.
## [0.2.2](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fleet@0.2.1...solverforge-fleet@0.2.2) (2026-11-10)


### Bug Fixes

* **runtime:** target SolverForge 0.25.1 across the bundle ([093fc61](https://github.com/SolverForge/solverforge-usecases/commit/093fc61c49983df6d6f0c0b3ba68408439a77869))


## [0.2.1](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fleet@0.2.0...solverforge-fleet@0.2.1) (2026-11-09)


### Bug Fixes

* **runtime:** target SolverForge 0.25.0 across the bundle ([0ffbf64](https://github.com/SolverForge/solverforge-usecases/commit/0ffbf64b997b7c80279202f5c1690c62b463146d))


## [0.2.0](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fleet@0.1.9...solverforge-fleet@0.2.0) (2026-11-03)


### ⚠ BREAKING CHANGES

* **analysis:** analysis constraints report a numeric `weightMultiplier` instead of the `weight` string, and flightcrew, furnace and orders selector telemetry no longer carries `conflictRepair*` counters.

### Features

* **analysis:** report the constraint weightMultiplier on SolverForge 0.24.0 ([e86f8ce](https://github.com/SolverForge/solverforge-usecases/commit/e86f8ce293997118faf42cc173b757f4b614bc1f))


### Bug Fixes

* **build:** make every app's browser tests run from a standalone clone ([2dab038](https://github.com/SolverForge/solverforge-usecases/commit/2dab038c4de559a6e80b0134fd16e86ff170f9dc))
* **build:** run slow acceptance solves in the release profile ([6c279ef](https://github.com/SolverForge/solverforge-usecases/commit/6c279eff404d49015f1f8dca6349492b54de9ea2))
* **fleet:** declare each constraint id with .named(...) ([7bcafab](https://github.com/SolverForge/solverforge-usecases/commit/7bcafab612f352d4f8be38c06cf4d1b23e0ebc6e))
* **runtime:** target SolverForge 0.24.0 across the bundle ([32c9e75](https://github.com/SolverForge/solverforge-usecases/commit/32c9e75a3462a07c0d4e87f5e677c76c5dd8adda))


## [0.1.9](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fleet@0.1.8...solverforge-fleet@0.1.9) (2026-10-27)


### Bug Fixes

* **e2e:** follow shared Cargo targets in remaining smoke servers ([10ba224](https://github.com/SolverForge/solverforge-usecases/commit/10ba2242ad374f410edb68c0429d437095249660))
* **runtime:** target SolverForge 0.23.0 across the bundle ([836d2b9](https://github.com/SolverForge/solverforge-usecases/commit/836d2b9d5b3d6ede6f69f2e6b6fc2920121314cb))


## [0.1.8](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fleet@solverforge-fleet@0.1.7...solverforge-fleet@solverforge-fleet@0.1.8) (2026-10-18)


### Bug Fixes

* **runtime:** target SolverForge 0.22.0 across the bundle ([36303f3](https://github.com/SolverForge/solverforge-usecases/commit/36303f3f059b78e5618f61ebb7c920c80c5c115f))


## [0.1.7](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fleet@solverforge-fleet@0.1.6...solverforge-fleet@solverforge-fleet@0.1.7) (2026-10-15)


### Bug Fixes

* **runtime:** target SolverForge 0.21.3 across the bundle ([9a87be4](https://github.com/SolverForge/solverforge-usecases/commit/9a87be40f5797a8c95d8b33b6af392497457a99a))


## [0.1.6](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fleet@solverforge-fleet@0.1.5...solverforge-fleet@solverforge-fleet@0.1.6) (2026-10-14)


### Bug Fixes

* **runtime:** target SolverForge 0.21.0 across the bundle ([0c9732b](https://github.com/SolverForge/solverforge-usecases/commit/0c9732bb9a4546adbd1859f2974c16ffe0551b41))
* **runtime:** target SolverForge 0.21.2 across the bundle ([7a614c3](https://github.com/SolverForge/solverforge-usecases/commit/7a614c34a6614c1d649edee6426cfd459400e03d))


## 0.1.5 (2026-09-27)


### Bug Fixes

* **fleet:** target SolverForge 0.19.7 and solverforge-ui 0.9.0 1c9ae81

## 0.1.4 (2026-09-25)


### Bug Fixes

* **ui:** include dock_outage in the fleet constraint model 8591a32

## 0.1.3 (2026-09-25)


### Bug Fixes

* **ui:** show hard/soft type in fleet score analysis 1a0fefd

## 0.1.2 (2026-09-18)


### Bug Fixes

* **fleet:** block starting a solve during a repair a24d6d0

## 0.1.1 (2026-09-18)


### Bug Fixes

* **fleet:** show repair progress and honor solver state cebb715

## 0.1.0 (2026-09-18)

### Features

* **fleet:** add synthetic fleet readiness planning app
