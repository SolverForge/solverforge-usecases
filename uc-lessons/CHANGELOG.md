# Changelog

All notable changes to this use case are documented in this file.
## [3.0.4](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-lessons@3.0.3...solverforge-lessons@3.0.4) (2026-11-11)


### Bug Fixes

* **runtime:** target SolverForge 0.26.1 across the bundle ([e88bc0e](https://github.com/SolverForge/solverforge-usecases/commit/e88bc0e2f9cee2d24d18be223dc38b740570061a)), closes #8


## [3.0.3](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-lessons@3.0.2...solverforge-lessons@3.0.3) (2026-11-10)


### Bug Fixes

* **runtime:** target SolverForge 0.26.0 across the bundle ([75f505b](https://github.com/SolverForge/solverforge-usecases/commit/75f505b31645023a6e22278534ec6d867ea17803))


## [3.0.2](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-lessons@3.0.1...solverforge-lessons@3.0.2) (2026-11-10)


### Bug Fixes

* **runtime:** target SolverForge 0.25.1 across the bundle ([093fc61](https://github.com/SolverForge/solverforge-usecases/commit/093fc61c49983df6d6f0c0b3ba68408439a77869))


## [3.0.1](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-lessons@3.0.0...solverforge-lessons@3.0.1) (2026-11-09)


### Bug Fixes

* **runtime:** target SolverForge 0.25.0 across the bundle ([0ffbf64](https://github.com/SolverForge/solverforge-usecases/commit/0ffbf64b997b7c80279202f5c1690c62b463146d))


## [3.0.0](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-lessons@2.0.14...solverforge-lessons@3.0.0) (2026-11-03)


### ⚠ BREAKING CHANGES

* **lessons:** /jobs/{id}/analysis reports constraint names as module ids (e.g. "late_lesson") instead of display labels.
* **analysis:** analysis constraints report a numeric `weightMultiplier` instead of the `weight` string, and flightcrew, furnace and orders selector telemetry no longer carries `conflictRepair*` counters.

### Features

* **analysis:** report the constraint weightMultiplier on SolverForge 0.24.0 ([e86f8ce](https://github.com/SolverForge/solverforge-usecases/commit/e86f8ce293997118faf42cc173b757f4b614bc1f))


### Bug Fixes

* **build:** make every app's browser tests run from a standalone clone ([2dab038](https://github.com/SolverForge/solverforge-usecases/commit/2dab038c4de559a6e80b0134fd16e86ff170f9dc))
* **build:** run slow acceptance solves in the release profile ([6c279ef](https://github.com/SolverForge/solverforge-usecases/commit/6c279eff404d49015f1f8dca6349492b54de9ea2))
* **lessons:** name constraints by module id and keep readable labels ([6f3a852](https://github.com/SolverForge/solverforge-usecases/commit/6f3a852b6800b7006941128a8c6274411896dea8))
* **runtime:** target SolverForge 0.24.0 across the bundle ([32c9e75](https://github.com/SolverForge/solverforge-usecases/commit/32c9e75a3462a07c0d4e87f5e677c76c5dd8adda))


## [2.0.14](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-lessons@2.0.13...solverforge-lessons@2.0.14) (2026-10-27)


### Bug Fixes

* **e2e:** follow shared Cargo targets in remaining smoke servers ([10ba224](https://github.com/SolverForge/solverforge-usecases/commit/10ba2242ad374f410edb68c0429d437095249660))
* **runtime:** target SolverForge 0.23.0 across the bundle ([836d2b9](https://github.com/SolverForge/solverforge-usecases/commit/836d2b9d5b3d6ede6f69f2e6b6fc2920121314cb))


## [2.0.13](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-lessons@solverforge-lessons@2.0.12...solverforge-lessons@solverforge-lessons@2.0.13) (2026-10-18)


### Bug Fixes

* **runtime:** target SolverForge 0.22.0 across the bundle ([36303f3](https://github.com/SolverForge/solverforge-usecases/commit/36303f3f059b78e5618f61ebb7c920c80c5c115f))


## [2.0.12](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-lessons@solverforge-lessons@2.0.11...solverforge-lessons@solverforge-lessons@2.0.12) (2026-10-15)


### Bug Fixes

* **runtime:** target SolverForge 0.21.3 across the bundle ([9a87be4](https://github.com/SolverForge/solverforge-usecases/commit/9a87be40f5797a8c95d8b33b6af392497457a99a))


## [2.0.11](https://github.com/SolverForge/solverforge-usecases/compare/solverforge-lessons@solverforge-lessons@2.0.10...solverforge-lessons@solverforge-lessons@2.0.11) (2026-10-14)


### Bug Fixes

* **runtime:** target SolverForge 0.21.0 across the bundle ([0c9732b](https://github.com/SolverForge/solverforge-usecases/commit/0c9732bb9a4546adbd1859f2974c16ffe0551b41))
* **runtime:** target SolverForge 0.21.2 across the bundle ([7a614c3](https://github.com/SolverForge/solverforge-usecases/commit/7a614c34a6614c1d649edee6426cfd459400e03d))


## 2.0.10 (2026-09-27)


### Bug Fixes

* **lessons:** target SolverForge 0.19.7 and solverforge-ui 0.9.0 d61a0e2

## 2.0.9 (2026-09-25)


### Bug Fixes

* **ui:** render the lessons score-analysis table as markup 93e8118

## 2.0.8 (2026-09-25)

## 2.0.7 (2026-08-13)

### Maintenance

* **release:** target SolverForge 0.19.4.

## 2.0.6 (2026-07-29)

### Maintenance

* **release:** target SolverForge 0.19.3.

## 2.0.5 (2026-07-17)


### Bug Fixes

* **lessons:** target SolverForge 0.19.0 a8e491a

## 2.0.4 (2026-07-13)

### Maintenance

* **release:** target SolverForge 0.18.0.
* **docs:** align score levels and browser demo loading with the current constraints and API.

## 2.0.3 (2026-06-16)

### Maintenance

* **release:** target SolverForge 0.17.1 and solverforge-cli 2.2.2.
* **metadata:** align generated scalar-variable metadata with the current SolverForge UI schema.

## 2.0.2 (2026-05-28)

### Maintenance

* **release:** target SolverForge 0.15.0.

## 2.0.1 (2026-05-16)

### Maintenance

* **release:** target SolverForge 0.14.1.

## 2.0.0 (2026-05-14)

### Maintenance

* **release:** set the public app release line to 2.0.0 across Cargo metadata and release validation.

## 0.1.0 (2026-05-14)

### Features

* **lessons:** publish the SolverForge lessons scheduling use case in the bundle.

### Maintenance

* **release:** align the bundled app with SolverForge 0.13.1 and solverforge-ui 0.6.5.
