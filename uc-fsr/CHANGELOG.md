# Changelog

All notable changes to this use case are documented in this file.
## [3.0.2](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fsr@3.0.1...solverforge-fsr@3.0.2) (2026-11-09)


### Bug Fixes

* **runtime:** target SolverForge 0.25.0 across the bundle ([0ffbf64](https://github.com/SolverForge/solverforge-usecases/commit/0ffbf64b997b7c80279202f5c1690c62b463146d))


## [3.0.1](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fsr@3.0.0...solverforge-fsr@3.0.1) (2026-11-03)


### Bug Fixes

* **routing:** identify the apps to Overpass with solverforge-maps 2.1.5 ([61d95ec](https://github.com/SolverForge/solverforge-usecases/commit/61d95ecabd67e1faa93dcaf209f0a7b0c01cf160))


## [3.0.0](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fsr@2.0.14...solverforge-fsr@3.0.0) (2026-11-03)


### ⚠ BREAKING CHANGES

* **fsr:** /jobs/{id}/analysis reports constraint names as snake_case ids (e.g. "time_windows") instead of display labels.
* **analysis:** analysis constraints report a numeric `weightMultiplier` instead of the `weight` string, and flightcrew, furnace and orders selector telemetry no longer carries `conflictRepair*` counters.

### Features

* **analysis:** report the constraint weightMultiplier on SolverForge 0.24.0 ([e86f8ce](https://github.com/SolverForge/solverforge-usecases/commit/e86f8ce293997118faf42cc173b757f4b614bc1f))


### Bug Fixes

* **build:** make every app's browser tests run from a standalone clone ([2dab038](https://github.com/SolverForge/solverforge-usecases/commit/2dab038c4de559a6e80b0134fd16e86ff170f9dc))
* **fsr:** name constraints by module id and keep readable labels ([d9302eb](https://github.com/SolverForge/solverforge-usecases/commit/d9302eb95d1b233702cb9c5050015672f3147bf2))
* **fsr:** return a JSON error body when the Overpass fetch fails ([ed40d12](https://github.com/SolverForge/solverforge-usecases/commit/ed40d121ad95f81ab005be3ce1e45a20a36e58ff))
* **runtime:** target SolverForge 0.24.0 across the bundle ([32c9e75](https://github.com/SolverForge/solverforge-usecases/commit/32c9e75a3462a07c0d4e87f5e677c76c5dd8adda))


## [2.0.14](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fsr@2.0.13...solverforge-fsr@2.0.14) (2026-10-27)


### Bug Fixes

* **e2e:** honor Cargo target directory in browser servers ([556cc7a](https://github.com/SolverForge/solverforge-usecases/commit/556cc7aedeff3eaaa821917f72f4f596c98eb52a))
* **runtime:** target SolverForge 0.23.0 across the bundle ([836d2b9](https://github.com/SolverForge/solverforge-usecases/commit/836d2b9d5b3d6ede6f69f2e6b6fc2920121314cb))


## [2.0.13](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fsr@solverforge-fsr@2.0.12...solverforge-fsr@solverforge-fsr@2.0.13) (2026-10-18)


### Bug Fixes

* **runtime:** target SolverForge 0.22.0 across the bundle ([36303f3](https://github.com/SolverForge/solverforge-usecases/commit/36303f3f059b78e5618f61ebb7c920c80c5c115f))


## [2.0.12](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fsr@solverforge-fsr@2.0.11...solverforge-fsr@solverforge-fsr@2.0.12) (2026-10-15)


### Bug Fixes

* **runtime:** target SolverForge 0.21.3 across the bundle ([9a87be4](https://github.com/SolverForge/solverforge-usecases/commit/9a87be40f5797a8c95d8b33b6af392497457a99a))


## [2.0.11](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-fsr@solverforge-fsr@2.0.10...solverforge-fsr@solverforge-fsr@2.0.11) (2026-10-14)


### Bug Fixes

* **runtime:** target SolverForge 0.21.0 across the bundle ([0c9732b](https://github.com/SolverForge/solverforge-usecases/commit/0c9732bb9a4546adbd1859f2974c16ffe0551b41))
* **runtime:** target SolverForge 0.21.2 across the bundle ([7a614c3](https://github.com/SolverForge/solverforge-usecases/commit/7a614c34a6614c1d649edee6426cfd459400e03d))


## 2.0.10 (2026-09-27)


### Bug Fixes

* **fsr:** target SolverForge 0.19.7 and solverforge-ui 0.9.0 2568778

## 2.0.9 (2026-09-25)


### Bug Fixes

* **ui:** show hard/soft type in FSR score analysis e37d030

## 2.0.8 (2026-08-13)

### Maintenance

* **release:** target SolverForge 0.19.4 and solverforge-core 0.19.4.

## 2.0.7 (2026-07-29)

### Maintenance

* **release:** target SolverForge 0.19.3 and solverforge-core 0.19.3.

## 2.0.6 (2026-07-17)


### Bug Fixes

* **fsr:** target SolverForge 0.19.0 da3db25

## 2.0.5 (2026-07-13)

### Maintenance

* **release:** target SolverForge 0.18.0 and solverforge-core 0.18.0.
* **docs:** align priority-slack semantics and the source map with the current app.
* **metadata:** use the canonical uppercase `STANDARD` demo id.

## 2.0.4 (2026-06-16)

### Maintenance

* **release:** target SolverForge 0.17.1, solverforge-core 0.17.1, and solverforge-cli 2.2.2.
* **metadata:** align generated list-variable metadata with the current SolverForge UI schema.

## 2.0.3 (2026-05-28)

### Maintenance

* **release:** target SolverForge 0.15.0 and solverforge-core 0.15.0.

## 2.0.2 (2026-05-28)

### Fixes

* **scoring:** rebuild transient visit indexes after JSON round trips and enforce exactly-once service visit assignment.

### Maintenance

* **docs:** align FSR architecture notes and use-case wireframes with the current retained runtime and API surface.

## 2.0.1 (2026-05-16)

### Maintenance

* **release:** target SolverForge 0.14.1 and solverforge-core 0.14.1.

## 2.0.0 (2026-05-14)

### Maintenance

* **release:** set the public app release line to 2.0.0 across Cargo metadata and release validation.

## 1.0.1 (2026-05-14)

### Features

* **fsr:** publish the SolverForge field-service routing use case in the bundle.

### Maintenance

* **release:** align the bundled app with SolverForge 0.13.1, solverforge-core 0.13.1, solverforge-ui 0.6.5, and solverforge-maps 2.1.4.
