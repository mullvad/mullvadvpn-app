// flow-typed signature: 88e8ce86d46701f2c1717cbba2890283
// flow-typed version: 19506e57e6/react-router_v4.x.x/flow_>=v0.30.x <=v0.52.x

declare module 'react-router' {
  // NOTE: many of these are re-exported by react-router-dom and
  // react-router-native, so when making changes, please be sure to update those
  // as well.
  declare export type Location = {
    pathname: string,
    search: string,
    hash: string,
    state?: any,
    key?: string,
  }

  declare export type LocationShape = {
    pathname?: string,
    search?: string,
    hash?: string,
    state?: any,
  }

  declare export type HistoryAction = 'PUSH' | 'REPLACE' | 'POP'

  declare export type RouterHistory = {
    length: number,
    location: Location,
    action: HistoryAction,
    listen(callback: (location: Location, action: HistoryAction) => void): () => void,
    push(path: string | LocationShape, state?: any): void,
    replace(path: string | LocationShape, state?: any): void,
    go(n: number): void,
    goBack(): void,
    goForward(): void,
    canGo?: (n: number) => bool,
    block(callback: (location: Location, action: HistoryAction) => boolean): void,
    // createMemoryHistory
    index?: number,
    entries?: Array<Location>,
  }

  declare export type Match = {
    params: { [key: string]: ?string },
    isExact: boolean,
    path: string,
    url: string,
  }

  declare export type ContextRouter = {
    history: RouterHistory,
    location: Location,
    match: Match,
  }

  declare export type GetUserConfirmation =
    (message: string, callback: (confirmed: boolean) => void) => void

  declare type StaticRouterContext = {
    url?: string,
  }

  declare export class StaticRouter extends React$Component {
    props: {
      basename?: string,
      location?: string | Location,
      context: StaticRouterContext,
      children?: React$Element<*>,
    }
  }

  declare export class MemoryRouter extends React$Component {
    props: {
      initialEntries?: Array<LocationShape | string>,
      initialIndex?: number,
      getUserConfirmation?: GetUserConfirmation,
      keyLength?: number,
      children?: React$Element<*>,
    }
  }

  declare export class Router extends React$Component {
    props: {
      history: RouterHistory,
      children?: React$Element<*>,
    }
  }

  declare export class Prompt extends React$Component {
    props: {
      message: string | (location: Location) => string | true,
      when?: boolean,
    }
  }

  declare export class Redirect extends React$Component {
    props: {
      to: string | LocationShape,
      push?: boolean,
    }
  }

  declare export class Route extends React$Component {
    props: {
      component?: ReactClass<*>,
      render?: (router: ContextRouter) => React$Element<*>,
      children?: (router: ContextRouter) => React$Element<*>,
      path?: string,
      exact?: bool,
      strict?: bool,
    }
  }

  declare export class Switch extends React$Component {
    props: {
      children?: Array<React$Element<*>>,
    }
  }

  declare type FunctionComponent<P> = (props: P) => ?React$Element<any>;
  declare type ClassComponent<D, P, S> = Class<React$Component<D, P, S>>;
  declare export function withRouter<D, P, S>(Component: ClassComponent<D, P, S> | FunctionComponent<P>): ClassComponent<D, $Diff<P, ContextRouter>, S>;

  declare type MatchPathOptions = {
    exact?: boolean,
    strict?: boolean,
  }
  declare export function matchPath(pathname: string, path: string, options?: MatchPathOptions): null | Match
}
