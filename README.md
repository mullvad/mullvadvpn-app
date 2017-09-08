# Mullvad VPN

## License

Copyright (C) 2017  Amagicom AB

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

## Developing

First you need to install all the javascript dependencies by running
```bash
yarn install
```
then you can start the program using
```bash
yarn run develop
```

If you change any javascript file while the development mode is running it will automatically transpile and reload the file so that the changes are visible almost immediately.

The app will attempt to start the backend automatically. The exact binary being run can be customized with the `MULLVAD_BACKEND` environment variable.

If the `/tmp/.mullvad_rpc_address` file exists the app will not start the backend, so if you want to run a specific version of the backend you can just start it yourself and the app will pick up on it and behave accordingly.


## Packaging

By running
```bash
yarn run pack
```
you create installation packages for windows, linux and MacOS. Note that you have to have run `yarn install` at least once to download the javascript dependencies.

If you only want to build for a specific OS you run
```bash
yarn run pack:OS
```
as in `yarn run pack:linux`.

The artifact (.dmg, .deb, .msi) version is the `version` property of `package.json`.

### Build dependencies

#### Linux

```bash
sudo apt install icnsutils graphicsmagick
```


## Command line tools

- `$ yarn run develop` - develop app with live-reload enabled
- `$ yarn run flow` - type-check the code
- `$ yarn run lint` - lint code
- `$ yarn run pack` - prepare app for distribution for macOS, Windows, Linux. Use `pack:mac`, `pack:win`, `pack:linux` to generate package for single target.
- `$ yarn run test` - run tests

## Structure

- **app/**
  - **redux/** - state management
  - **components/** - components
  - **containers/** - containers that provide a glueing layer between components and redux actions/backend.
  - **lib/** - shared classes and utilities
  - **assets/** - graphical assets and stylesheets
  - **config.js** - static configuration file
  - **app.js** - entry file for renderer process
  - **main.js** - entry file for background process
  - **routes.js** - routes configurator
  - **store.js** - redux store configurator
  - **enums.js** - common enums used across components
  - **tilecache.sw.js** - service worker for caching mapbox requests
- **test/** - tests
- **scripts/** - support scripts for development
- **init.js** - entry file for electron, points to compiled **main.js**

## App diagram

![App diagram](README%20images/app-diagram.png)

## View layout

Most of application layouts consist of header bar area and main content area. Three of components from `components/Layout` help to assemble each view, i.e:

```
<Layout>
  <Header />
  <Container>
    { /* content goes here */ }
  </Container>
</Layout>
```
