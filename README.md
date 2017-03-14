# Mullvad VPN

## Command line tools

- `$ npm run develop` - develop app with live-reload enabled
- `$ npm run lint` - lint code
- `$ npm run docs` - generate HTML documentation
- `$ npm run pack` - prepare app for distribution for macOS, Windows, Linux. Use `pack:mac`, `pack:win`, `pack:linux` to generate package for single target.
- `$ npm run test` - run tests

## Structure

- **app/**
  - **actions/** - redux actions
  - **reducers/** - redux reducers
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
- **distribution.yml** - distribution configuration

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

## Backend communication

Backend is connected with the app via event system.

Backend events are translated into redux actions. There are two helpers responsible for events translation:

- **app/lib/backend-redux-actions.js** - translates events into redux actions
- **app/lib/backend-routing.js** - application logic based on received events (i.e managing active route etc..)
