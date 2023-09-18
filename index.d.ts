/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export class Surreal {
  constructor()
  connect(endpoint: string, opts?: any | undefined | null): Promise<void>
  use(value: any): Promise<void>
  set(key: string, value: any): Promise<void>
  unset(key: string): Promise<void>
  signup(credentials: any): Promise<any>
  signin(credentials: any): Promise<any>
  invalidate(): Promise<void>
  authenticate(token: string): Promise<void>
  query(sql: string, bindings?: any | undefined | null): Promise<any>
  select(resource: string): Promise<any>
  create(resource: string, data?: any | undefined | null): Promise<any>
  update(resource: string, data: any): Promise<any>
  merge(resource: string, data: any): Promise<any>
  patch(resource: string, data: any): Promise<any>
  delete(resource: string): Promise<any>
  version(): Promise<any>
  health(): Promise<void>
}