# Changelog

All notable changes to this use case are documented in this file.
## [3.0.2](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-deliveries@3.0.1...solverforge-deliveries@3.0.2) (2026-11-09)


### Bug Fixes

* **runtime:** target SolverForge 0.25.0 across the bundle ([0ffbf64](https://github.com/SolverForge/solverforge-usecases/commit/0ffbf64b997b7c80279202f5c1690c62b463146d))


## [3.0.1](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-deliveries@3.0.0...solverforge-deliveries@3.0.1) (2026-11-03)


### Bug Fixes

* **routing:** identify the apps to Overpass with solverforge-maps 2.1.5 ([61d95ec](https://github.com/SolverForge/solverforge-usecases/commit/61d95ecabd67e1faa93dcaf209f0a7b0c01cf160))


## [3.0.0](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-deliveries@2.0.13...solverforge-deliveries@3.0.0) (2026-11-03)


### ⚠ BREAKING CHANGES

* **deliveries:** /jobs/{id}/analysis reports constraint names as module ids (e.g. "vehicle_capacity") instead of display labels.
* **analysis:** analysis constraints report a numeric `weightMultiplier` instead of the `weight` string, and flightcrew, furnace and orders selector telemetry no longer carries `conflictRepair*` counters.

### Features

* **analysis:** report the constraint weightMultiplier on SolverForge 0.24.0 ([e86f8ce](https://github.com/SolverForge/solverforge-usecases/commit/e86f8ce293997118faf42cc173b757f4b614bc1f))


### Bug Fixes

* **build:** make every app's browser tests run from a standalone clone ([2dab038](https://github.com/SolverForge/solverforge-usecases/commit/2dab038c4de559a6e80b0134fd16e86ff170f9dc))
* **deliveries:** bound the live road-network lifecycle polls ([fd3c4e8](https://github.com/SolverForge/solverforge-usecases/commit/fd3c4e81e678e4dd2a37635d030a82f2896b82f7))
* **deliveries:** name the cause of road-network routing failures ([55bb08d](https://github.com/SolverForge/solverforge-usecases/commit/55bb08d8edff80175058a388c806785c73229db3))
* **deliveries:** pass solverforge check and keep readable constraint labels ([e372ed7](https://github.com/SolverForge/solverforge-usecases/commit/e372ed75c90136232e6f6d82dd32d26404432230))
* **runtime:** target SolverForge 0.24.0 across the bundle ([32c9e75](https://github.com/SolverForge/solverforge-usecases/commit/32c9e75a3462a07c0d4e87f5e677c76c5dd8adda))


## [2.0.13](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-deliveries@2.0.12...solverforge-deliveries@2.0.13) (2026-10-27)


### Bug Fixes

* **e2e:** honor Cargo target directory in browser servers ([556cc7a](https://github.com/SolverForge/solverforge-usecases/commit/556cc7aedeff3eaaa821917f72f4f596c98eb52a))
* **runtime:** target SolverForge 0.23.0 across the bundle ([836d2b9](https://github.com/SolverForge/solverforge-usecases/commit/836d2b9d5b3d6ede6f69f2e6b6fc2920121314cb))


## [2.0.12](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-deliveries@solverforge-deliveries@2.0.11...solverforge-deliveries@solverforge-deliveries@2.0.12) (2026-10-18)


### Bug Fixes

* **runtime:** target SolverForge 0.22.0 across the bundle ([36303f3](https://github.com/SolverForge/solverforge-usecases/commit/36303f3f059b78e5618f61ebb7c920c80c5c115f))


## [2.0.11](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-deliveries@solverforge-deliveries@2.0.10...solverforge-deliveries@solverforge-deliveries@2.0.11) (2026-10-15)


### Bug Fixes

* **runtime:** target SolverForge 0.21.3 across the bundle ([9a87be4](https://github.com/SolverForge/solverforge-usecases/commit/9a87be40f5797a8c95d8b33b6af392497457a99a))


## [2.0.10](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-deliveries@solverforge-deliveries@2.0.9...solverforge-deliveries@solverforge-deliveries@2.0.10) (2026-10-14)


### Bug Fixes

* **runtime:** target SolverForge 0.21.0 across the bundle ([0c9732b](https://github.com/SolverForge/solverforge-usecases/commit/0c9732bb9a4546adbd1859f2974c16ffe0551b41))
* **runtime:** target SolverForge 0.21.2 across the bundle ([7a614c3](https://github.com/SolverForge/solverforge-usecases/commit/7a614c34a6614c1d649edee6426cfd459400e03d))


## 2.0.9 (2026-09-27)


### Bug Fixes

* **deliveries:** target SolverForge 0.19.7 and solverforge-ui 0.9.0 000a9ae

## 2.0.8 (2026-09-25)


### Bug Fixes

* **ui:** show hard/soft type in deliveries score analysis 54979dd

## 2.0.7 (2026-08-13)

### Maintenance

* **release:** target SolverForge 0.19.4.

## 2.0.6 (2026-07-29)

### Maintenance

* **release:** target SolverForge 0.19.3.

## 2.0.5 (2026-07-17)


### Bug Fixes

* **deliveries:** target SolverForge 0.19.0 ace9700

## 2.0.4 (2026-07-13)

### Maintenance

* **release:** target SolverForge 0.18.0.
* **tests:** run custom construction configs through the compiled runtime entrypoint.
* **docs:** name the shipped local-search forager exactly as configured.

## 2.0.3 (2026-06-16)

### Features

* **routing:** use the road-network-only Deliveries contract with stock SolverForge CVRP construction hooks.

### Maintenance

* **release:** target SolverForge 0.17.1 and solverforge-cli 2.2.2.

## 2.0.2 (2026-05-28)

### Maintenance

* **release:** target SolverForge 0.15.0.

## 2.0.1 (2026-05-16)

### Maintenance

* **release:** target SolverForge 0.14.1 and migrate delivery route hooks to the owner-aware 0.14 list-variable API.

## 2.0.0 (2026-05-14)

### Maintenance

* **release:** set the public app release line to 2.0.0 across Cargo metadata and release validation.

## 1.0.1 (2026-05-14)

### Features

* **deliveries:** publish the SolverForge deliveries use case in the bundle.

### Maintenance

* **release:** align the bundled app with SolverForge 0.13.1, solverforge-ui 0.6.5, and solverforge-maps 2.1.4.
