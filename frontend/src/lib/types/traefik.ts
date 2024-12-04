// To parse this data:
//
//   import { Convert, DeploymentSchema, FromClientIPSchema, HeadersSchema, HostSchema, MiddlewareSchema, PathSchema, SelectionSchema, TraefikSchema, TraefikctlConfigDeploymentDeploymentConfigSchema, TraefikctlConfigHeadersHeadersConfigSchema, TraefikctlConfigHostHostConfigSchema, TraefikctlConfigHostPathConfigSchema, TraefikctlConfigMiddlewareMiddlewareConfigSchema, TraefikctlConfigSelectionsFromClientIPConfigSchema, TraefikctlConfigSelectionsSelectionConfigSchema, TraefikctlConfigSelectionsWithCookieConfigSchema, TraefikctlConfigTraefikConfigTraefikConfigSchema, WithCookieSchema } from "./file";
//
//   const deploymentSchema = Convert.toDeploymentSchema(json);
//   const fromClientIPSchema = Convert.toFromClientIPSchema(json);
//   const headersSchema = Convert.toHeadersSchema(json);
//   const hostSchema = Convert.toHostSchema(json);
//   const middlewareSchema = Convert.toMiddlewareSchema(json);
//   const pathSchema = Convert.toPathSchema(json);
//   const selectionSchema = Convert.toSelectionSchema(json);
//   const traefikSchema = Convert.toTraefikSchema(json);
//   const traefikctlConfigDeploymentDeploymentConfigSchema = Convert.toTraefikctlConfigDeploymentDeploymentConfigSchema(json);
//   const traefikctlConfigHeadersHeadersConfigSchema = Convert.toTraefikctlConfigHeadersHeadersConfigSchema(json);
//   const traefikctlConfigHostHostConfigSchema = Convert.toTraefikctlConfigHostHostConfigSchema(json);
//   const traefikctlConfigHostPathConfigSchema = Convert.toTraefikctlConfigHostPathConfigSchema(json);
//   const traefikctlConfigMiddlewareMiddlewareConfigSchema = Convert.toTraefikctlConfigMiddlewareMiddlewareConfigSchema(json);
//   const traefikctlConfigSelectionsFromClientIPConfigSchema = Convert.toTraefikctlConfigSelectionsFromClientIPConfigSchema(json);
//   const traefikctlConfigSelectionsSelectionConfigSchema = Convert.toTraefikctlConfigSelectionsSelectionConfigSchema(json);
//   const traefikctlConfigSelectionsWithCookieConfigSchema = Convert.toTraefikctlConfigSelectionsWithCookieConfigSchema(json);
//   const traefikctlConfigTraefikConfigTraefikConfigSchema = Convert.toTraefikctlConfigTraefikConfigTraefikConfigSchema(json);
//   const withCookieSchema = Convert.toWithCookieSchema(json);
//
// These functions will throw an error if the JSON doesn't
// match the expected interface, even if the JSON is valid.

export interface DeploymentSchema {
    $schema:     string;
    $ref:        string;
    definitions: DeploymentSchemaDefinitions;
}

export interface DeploymentSchemaDefinitions {
    DeploymentSchema: Schema;
    Definitions:      Definitions;
    Properties:       PurpleProperties;
    IP:               AddVaryHeaderClass;
}

export interface Definitions {
    type:                 string;
    additionalProperties: boolean;
    title:                string;
}

export interface Schema {
    type:                 string;
    additionalProperties: boolean;
    properties:           DeploymentSchemaProperties;
    required:             Required[];
    title:                string;
}

export interface DeploymentSchemaProperties {
    $schema:     SchemaClass;
    definitions: IPClass;
    properties:  IPClass;
    title:       NameClass;
    type:        NameClass;
}

export interface SchemaClass {
    type:               Type;
    format:             string;
    "qt-uri-protocols": string[];
}

export enum Type {
    Boolean = "boolean",
    Integer = "integer",
    String = "string",
}

export interface IPClass {
    $ref: string;
}

export interface NameClass {
    type: Type;
}

export enum Required {
    Definitions = "definitions",
    Properties = "properties",
    Schema = "$schema",
    Title = "title",
    Type = "type",
}

export interface AddVaryHeaderClass {
    type:                 string;
    additionalProperties: boolean;
    properties:           IPProperties;
    required:             Required[];
    title:                string;
}

export interface IPProperties {
    type: NameClass;
}

export interface PurpleProperties {
    type:                 string;
    additionalProperties: boolean;
    properties:           FluffyProperties;
    required:             string[];
    title:                string;
}

export interface FluffyProperties {
    ip:       IPClass;
    name:     IPClass;
    port:     IPClass;
    protocol: NameClass;
    weight:   IPClass;
}

export interface FromClientIPSchema {
    $schema:     string;
    $ref:        string;
    definitions: FromClientIPSchemaDefinitions;
}

export interface FromClientIPSchemaDefinitions {
    FromClientIPSchema: Schema;
    Definitions:        Definitions;
    Properties:         TentacledProperties;
}

export interface TentacledProperties {
    type:                 string;
    additionalProperties: boolean;
    properties:           StickyProperties;
    required:             string[];
    title:                string;
}

export interface StickyProperties {
    ip:    NameClass;
    range: NameClass;
}

export interface HeadersSchema {
    $schema:     string;
    $ref:        string;
    definitions: HeadersSchemaDefinitions;
}

export interface HeadersSchemaDefinitions {
    HeadersSchema:   Schema;
    Definitions:     Definitions;
    Properties:      IndigoProperties;
    AccessControl:   AccessControl;
    AddVaryHeader:   AddVaryHeaderClass;
    CustomReHeaders: CustomReHeaders;
}

export interface AccessControl {
    type:                 string;
    additionalProperties: boolean;
    properties:           AccessControlProperties;
    required:             string[];
    title:                string;
}

export interface AccessControlProperties {
    items: NameClass;
    type:  NameClass;
}

export interface CustomReHeaders {
    type:                 string;
    additionalProperties: boolean;
    properties:           CustomReHeadersProperties;
    required:             string[];
    title:                string;
}

export interface CustomReHeadersProperties {
    additionalProperties: NameClass;
    type:                 NameClass;
}

export interface IndigoProperties {
    type:                 string;
    additionalProperties: boolean;
    properties:           IndecentProperties;
    required:             string[];
    title:                string;
}

export interface IndecentProperties {
    accessControlAllowHeaders:    IPClass;
    accessControlAllowMethods:    IPClass;
    accessControlAllowOriginList: IPClass;
    accessControlExposeHeaders:   IPClass;
    addVaryHeader:                IPClass;
    customRequestHeaders:         IPClass;
    customResponseHeaders:        IPClass;
}

export interface HostSchema {
    $schema:     string;
    $ref:        string;
    definitions: HostSchemaDefinitions;
}

export interface HostSchemaDefinitions {
    HostSchema:  Schema;
    Definitions: Definitions;
    Properties:  HilariousProperties;
    Deployments: CustomReHeaders;
    Domain:      AddVaryHeaderClass;
    Middlewares: AccessControl;
}

export interface HilariousProperties {
    type:                 string;
    additionalProperties: boolean;
    properties:           AmbitiousProperties;
    required:             string[];
    title:                string;
}

export interface AmbitiousProperties {
    deployments: IPClass;
    domain:      IPClass;
    middlewares: IPClass;
    paths:       IPClass;
}

export interface MiddlewareSchema {
    $schema:     string;
    $ref:        string;
    definitions: MiddlewareSchemaDefinitions;
}

export interface MiddlewareSchemaDefinitions {
    MiddlewareSchema: Schema;
    Definitions:      Definitions;
    Properties:       CunningProperties;
    Name:             AddVaryHeaderClass;
}

export interface CunningProperties {
    type:                 string;
    additionalProperties: boolean;
    properties:           MagentaProperties;
    required:             string[];
    title:                string;
}

export interface MagentaProperties {
    name:     IPClass;
    protocol: IPClass;
}

export interface PathSchema {
    $schema:     string;
    $ref:        string;
    definitions: PathSchemaDefinitions;
}

export interface PathSchemaDefinitions {
    PathSchema:  Schema;
    Definitions: Definitions;
    Properties:  FriskyProperties;
    Deployments: CustomReHeaders;
    Middlewares: AccessControl;
    PassThrough: AddVaryHeaderClass;
}

export interface FriskyProperties {
    type:                 string;
    additionalProperties: boolean;
    properties:           MischievousProperties;
    required:             string[];
    title:                string;
}

export interface MischievousProperties {
    deployments: IPClass;
    middlewares: IPClass;
    passThrough: IPClass;
    path:        IPClass;
    stripPrefix: IPClass;
}

export interface SelectionSchema {
    $schema:     string;
    $ref:        string;
    definitions: SelectionSchemaDefinitions;
}

export interface SelectionSchemaDefinitions {
    SelectionSchema: Schema;
    Definitions:     Definitions;
}

export interface TraefikSchema {
    $schema:     string;
    $ref:        string;
    definitions: TraefikSchemaDefinitions;
}

export interface TraefikSchemaDefinitions {
    TraefikSchema:           Schema;
    Definitions:             Definitions;
    TraefikSchemaProperties: TraefikSchemaProperties;
    Etcd:                    Etcd;
    EtcdProperties:          DefinitionsEtcdProperties;
    Endpoints:               Endpoints;
    RulePrefix:              AddVaryHeaderClass;
    Hosts:                   AccessControl;
    Middlewares:             CustomReHeaders;
}

export interface Endpoints {
    type:                 string;
    additionalProperties: boolean;
    properties:           EndpointsProperties;
    required:             string[];
    title:                string;
}

export interface EndpointsProperties {
    items: IPClass;
    type:  NameClass;
}

export interface Etcd {
    type:                 string;
    additionalProperties: boolean;
    properties:           EtcdProperties;
    required:             Required[];
    title:                string;
}

export interface EtcdProperties {
    properties: IPClass;
    type:       NameClass;
}

export interface DefinitionsEtcdProperties {
    type:                 string;
    additionalProperties: boolean;
    properties:           EtcdPropertiesProperties;
    required:             string[];
    title:                string;
}

export interface EtcdPropertiesProperties {
    endpoints:  IPClass;
    keep_alive: IPClass;
    timeout:    IPClass;
}

export interface TraefikSchemaProperties {
    type:                 string;
    additionalProperties: boolean;
    properties:           TraefikSchemaPropertiesProperties;
    required:             string[];
    title:                string;
}

export interface TraefikSchemaPropertiesProperties {
    etcd:        IPClass;
    hosts:       IPClass;
    middlewares: IPClass;
    rulePrefix:  IPClass;
}

export interface TraefikctlConfigDeploymentDeploymentConfigSchema {
    $schema:     string;
    definitions: TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions;
    properties:  TraefikctlConfigDeploymentDeploymentConfigSchemaProperties;
    title:       string;
    type:        string;
}

export interface TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions {
}

export interface TraefikctlConfigDeploymentDeploymentConfigSchemaProperties {
    ip:       NameClass;
    name:     NameClass;
    port:     NameClass;
    protocol: boolean;
    weight:   NameClass;
}

export interface TraefikctlConfigHeadersHeadersConfigSchema {
    $schema:     string;
    definitions: TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions;
    properties:  TraefikctlConfigHeadersHeadersConfigSchemaProperties;
    title:       string;
    type:        string;
}

export interface TraefikctlConfigHeadersHeadersConfigSchemaProperties {
    accessControlAllowHeaders:    AccessControlAllowHeaders;
    accessControlAllowMethods:    AccessControlAllowHeaders;
    accessControlAllowOriginList: AccessControlAllowHeaders;
    accessControlExposeHeaders:   AccessControlAllowHeaders;
    addVaryHeader:                NameClass;
    customRequestHeaders:         CustomRequestHeaders;
    customResponseHeaders:        CustomRequestHeaders;
}

export interface AccessControlAllowHeaders {
    items: boolean;
    type:  string;
}

export interface CustomRequestHeaders {
    additionalProperties: boolean;
    type:                 string;
}

export interface TraefikctlConfigHostHostConfigSchema {
    $schema:     string;
    definitions: TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions;
    properties:  TraefikctlConfigHostHostConfigSchemaProperties;
    title:       string;
    type:        string;
}

export interface TraefikctlConfigHostHostConfigSchemaProperties {
    deployments: CustomRequestHeaders;
    domain:      NameClass;
    middlewares: AccessControlAllowHeaders;
    paths:       AccessControlAllowHeaders;
}

export interface TraefikctlConfigHostPathConfigSchema {
    $schema:     string;
    definitions: TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions;
    properties:  TraefikctlConfigHostPathConfigSchemaProperties;
    title:       string;
    type:        string;
}

export interface TraefikctlConfigHostPathConfigSchemaProperties {
    deployments: CustomRequestHeaders;
    middlewares: AccessControlAllowHeaders;
    passThrough: NameClass;
    path:        NameClass;
    stripPrefix: NameClass;
}

export interface TraefikctlConfigMiddlewareMiddlewareConfigSchema {
    $schema:     string;
    definitions: TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions;
    properties:  TraefikctlConfigMiddlewareMiddlewareConfigSchemaProperties;
    title:       string;
    type:        string;
}

export interface TraefikctlConfigMiddlewareMiddlewareConfigSchemaProperties {
    name:     NameClass;
    protocol: NameClass;
}

export interface TraefikctlConfigSelectionsFromClientIPConfigSchema {
    $schema:     string;
    definitions: TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions;
    properties:  TraefikctlConfigSelectionsFromClientIPConfigSchemaProperties;
    title:       string;
    type:        string;
}

export interface TraefikctlConfigSelectionsFromClientIPConfigSchemaProperties {
    ip:    boolean;
    range: boolean;
}

export interface TraefikctlConfigSelectionsSelectionConfigSchema {
    $schema:     string;
    definitions: TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions;
    properties:  TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions;
    title:       string;
    type:        string;
}

export interface TraefikctlConfigSelectionsWithCookieConfigSchema {
    $schema:     string;
    definitions: TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions;
    properties:  TraefikctlConfigSelectionsWithCookieConfigSchemaProperties;
    title:       string;
    type:        string;
}

export interface TraefikctlConfigSelectionsWithCookieConfigSchemaProperties {
    name:  NameClass;
    value: boolean;
}

export interface TraefikctlConfigTraefikConfigTraefikConfigSchema {
    $schema:     string;
    definitions: TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions;
    properties:  TraefikctlConfigTraefikConfigTraefikConfigSchemaProperties;
    title:       string;
    type:        string;
}

export interface TraefikctlConfigTraefikConfigTraefikConfigSchemaProperties {
    etcd:        EtcdClass;
    hosts:       AccessControlAllowHeaders;
    middlewares: CustomRequestHeaders;
    rulePrefix:  NameClass;
}

export interface EtcdClass {
    properties: EtcdPropertiesClass;
    type:       string;
}

export interface EtcdPropertiesClass {
    endpoints:  EndpointsClass;
    keep_alive: NameClass;
    timeout:    NameClass;
}

export interface EndpointsClass {
    items: NameClass;
    type:  string;
}

export interface WithCookieSchema {
    $schema:     string;
    $ref:        string;
    definitions: WithCookieSchemaDefinitions;
}

export interface WithCookieSchemaDefinitions {
    WithCookieSchema: Schema;
    Definitions:      Definitions;
    Properties:       BraggadociousProperties;
    Name:             AddVaryHeaderClass;
}

export interface BraggadociousProperties {
    type:                 string;
    additionalProperties: boolean;
    properties:           Properties1;
    required:             string[];
    title:                string;
}

export interface Properties1 {
    name:  IPClass;
    value: NameClass;
}

// Converts JSON strings to/from your types
// and asserts the results of JSON.parse at runtime
export class Convert {
    public static toDeploymentSchema(json: string): DeploymentSchema {
        return cast(JSON.parse(json), r("DeploymentSchema"));
    }

    public static deploymentSchemaToJson(value: DeploymentSchema): string {
        return JSON.stringify(uncast(value, r("DeploymentSchema")), null, 2);
    }

    public static toFromClientIPSchema(json: string): FromClientIPSchema {
        return cast(JSON.parse(json), r("FromClientIPSchema"));
    }

    public static fromClientIPSchemaToJson(value: FromClientIPSchema): string {
        return JSON.stringify(uncast(value, r("FromClientIPSchema")), null, 2);
    }

    public static toHeadersSchema(json: string): HeadersSchema {
        return cast(JSON.parse(json), r("HeadersSchema"));
    }

    public static headersSchemaToJson(value: HeadersSchema): string {
        return JSON.stringify(uncast(value, r("HeadersSchema")), null, 2);
    }

    public static toHostSchema(json: string): HostSchema {
        return cast(JSON.parse(json), r("HostSchema"));
    }

    public static hostSchemaToJson(value: HostSchema): string {
        return JSON.stringify(uncast(value, r("HostSchema")), null, 2);
    }

    public static toMiddlewareSchema(json: string): MiddlewareSchema {
        return cast(JSON.parse(json), r("MiddlewareSchema"));
    }

    public static middlewareSchemaToJson(value: MiddlewareSchema): string {
        return JSON.stringify(uncast(value, r("MiddlewareSchema")), null, 2);
    }

    public static toPathSchema(json: string): PathSchema {
        return cast(JSON.parse(json), r("PathSchema"));
    }

    public static pathSchemaToJson(value: PathSchema): string {
        return JSON.stringify(uncast(value, r("PathSchema")), null, 2);
    }

    public static toSelectionSchema(json: string): SelectionSchema {
        return cast(JSON.parse(json), r("SelectionSchema"));
    }

    public static selectionSchemaToJson(value: SelectionSchema): string {
        return JSON.stringify(uncast(value, r("SelectionSchema")), null, 2);
    }

    public static toTraefikSchema(json: string): TraefikSchema {
        return cast(JSON.parse(json), r("TraefikSchema"));
    }

    public static traefikSchemaToJson(value: TraefikSchema): string {
        return JSON.stringify(uncast(value, r("TraefikSchema")), null, 2);
    }

    public static toTraefikctlConfigDeploymentDeploymentConfigSchema(json: string): TraefikctlConfigDeploymentDeploymentConfigSchema {
        return cast(JSON.parse(json), r("TraefikctlConfigDeploymentDeploymentConfigSchema"));
    }

    public static traefikctlConfigDeploymentDeploymentConfigSchemaToJson(value: TraefikctlConfigDeploymentDeploymentConfigSchema): string {
        return JSON.stringify(uncast(value, r("TraefikctlConfigDeploymentDeploymentConfigSchema")), null, 2);
    }

    public static toTraefikctlConfigHeadersHeadersConfigSchema(json: string): TraefikctlConfigHeadersHeadersConfigSchema {
        return cast(JSON.parse(json), r("TraefikctlConfigHeadersHeadersConfigSchema"));
    }

    public static traefikctlConfigHeadersHeadersConfigSchemaToJson(value: TraefikctlConfigHeadersHeadersConfigSchema): string {
        return JSON.stringify(uncast(value, r("TraefikctlConfigHeadersHeadersConfigSchema")), null, 2);
    }

    public static toTraefikctlConfigHostHostConfigSchema(json: string): TraefikctlConfigHostHostConfigSchema {
        return cast(JSON.parse(json), r("TraefikctlConfigHostHostConfigSchema"));
    }

    public static traefikctlConfigHostHostConfigSchemaToJson(value: TraefikctlConfigHostHostConfigSchema): string {
        return JSON.stringify(uncast(value, r("TraefikctlConfigHostHostConfigSchema")), null, 2);
    }

    public static toTraefikctlConfigHostPathConfigSchema(json: string): TraefikctlConfigHostPathConfigSchema {
        return cast(JSON.parse(json), r("TraefikctlConfigHostPathConfigSchema"));
    }

    public static traefikctlConfigHostPathConfigSchemaToJson(value: TraefikctlConfigHostPathConfigSchema): string {
        return JSON.stringify(uncast(value, r("TraefikctlConfigHostPathConfigSchema")), null, 2);
    }

    public static toTraefikctlConfigMiddlewareMiddlewareConfigSchema(json: string): TraefikctlConfigMiddlewareMiddlewareConfigSchema {
        return cast(JSON.parse(json), r("TraefikctlConfigMiddlewareMiddlewareConfigSchema"));
    }

    public static traefikctlConfigMiddlewareMiddlewareConfigSchemaToJson(value: TraefikctlConfigMiddlewareMiddlewareConfigSchema): string {
        return JSON.stringify(uncast(value, r("TraefikctlConfigMiddlewareMiddlewareConfigSchema")), null, 2);
    }

    public static toTraefikctlConfigSelectionsFromClientIPConfigSchema(json: string): TraefikctlConfigSelectionsFromClientIPConfigSchema {
        return cast(JSON.parse(json), r("TraefikctlConfigSelectionsFromClientIPConfigSchema"));
    }

    public static traefikctlConfigSelectionsFromClientIPConfigSchemaToJson(value: TraefikctlConfigSelectionsFromClientIPConfigSchema): string {
        return JSON.stringify(uncast(value, r("TraefikctlConfigSelectionsFromClientIPConfigSchema")), null, 2);
    }

    public static toTraefikctlConfigSelectionsSelectionConfigSchema(json: string): TraefikctlConfigSelectionsSelectionConfigSchema {
        return cast(JSON.parse(json), r("TraefikctlConfigSelectionsSelectionConfigSchema"));
    }

    public static traefikctlConfigSelectionsSelectionConfigSchemaToJson(value: TraefikctlConfigSelectionsSelectionConfigSchema): string {
        return JSON.stringify(uncast(value, r("TraefikctlConfigSelectionsSelectionConfigSchema")), null, 2);
    }

    public static toTraefikctlConfigSelectionsWithCookieConfigSchema(json: string): TraefikctlConfigSelectionsWithCookieConfigSchema {
        return cast(JSON.parse(json), r("TraefikctlConfigSelectionsWithCookieConfigSchema"));
    }

    public static traefikctlConfigSelectionsWithCookieConfigSchemaToJson(value: TraefikctlConfigSelectionsWithCookieConfigSchema): string {
        return JSON.stringify(uncast(value, r("TraefikctlConfigSelectionsWithCookieConfigSchema")), null, 2);
    }

    public static toTraefikctlConfigTraefikConfigTraefikConfigSchema(json: string): TraefikctlConfigTraefikConfigTraefikConfigSchema {
        return cast(JSON.parse(json), r("TraefikctlConfigTraefikConfigTraefikConfigSchema"));
    }

    public static traefikctlConfigTraefikConfigTraefikConfigSchemaToJson(value: TraefikctlConfigTraefikConfigTraefikConfigSchema): string {
        return JSON.stringify(uncast(value, r("TraefikctlConfigTraefikConfigTraefikConfigSchema")), null, 2);
    }

    public static toWithCookieSchema(json: string): WithCookieSchema {
        return cast(JSON.parse(json), r("WithCookieSchema"));
    }

    public static withCookieSchemaToJson(value: WithCookieSchema): string {
        return JSON.stringify(uncast(value, r("WithCookieSchema")), null, 2);
    }
}

function invalidValue(typ: any, val: any, key: any, parent: any = ''): never {
    const prettyTyp = prettyTypeName(typ);
    const parentText = parent ? ` on ${parent}` : '';
    const keyText = key ? ` for key "${key}"` : '';
    throw Error(`Invalid value${keyText}${parentText}. Expected ${prettyTyp} but got ${JSON.stringify(val)}`);
}

function prettyTypeName(typ: any): string {
    if (Array.isArray(typ)) {
        if (typ.length === 2 && typ[0] === undefined) {
            return `an optional ${prettyTypeName(typ[1])}`;
        } else {
            return `one of [${typ.map(a => { return prettyTypeName(a); }).join(", ")}]`;
        }
    } else if (typeof typ === "object" && typ.literal !== undefined) {
        return typ.literal;
    } else {
        return typeof typ;
    }
}

function jsonToJSProps(typ: any): any {
    if (typ.jsonToJS === undefined) {
        const map: any = {};
        typ.props.forEach((p: any) => map[p.json] = { key: p.js, typ: p.typ });
        typ.jsonToJS = map;
    }
    return typ.jsonToJS;
}

function jsToJSONProps(typ: any): any {
    if (typ.jsToJSON === undefined) {
        const map: any = {};
        typ.props.forEach((p: any) => map[p.js] = { key: p.json, typ: p.typ });
        typ.jsToJSON = map;
    }
    return typ.jsToJSON;
}

function transform(val: any, typ: any, getProps: any, key: any = '', parent: any = ''): any {
    function transformPrimitive(typ: string, val: any): any {
        if (typeof typ === typeof val) return val;
        return invalidValue(typ, val, key, parent);
    }

    function transformUnion(typs: any[], val: any): any {
        // val must validate against one typ in typs
        const l = typs.length;
        for (let i = 0; i < l; i++) {
            const typ = typs[i];
            try {
                return transform(val, typ, getProps);
            } catch (_) {}
        }
        return invalidValue(typs, val, key, parent);
    }

    function transformEnum(cases: string[], val: any): any {
        if (cases.indexOf(val) !== -1) return val;
        return invalidValue(cases.map(a => { return l(a); }), val, key, parent);
    }

    function transformArray(typ: any, val: any): any {
        // val must be an array with no invalid elements
        if (!Array.isArray(val)) return invalidValue(l("array"), val, key, parent);
        return val.map(el => transform(el, typ, getProps));
    }

    function transformDate(val: any): any {
        if (val === null) {
            return null;
        }
        const d = new Date(val);
        if (isNaN(d.valueOf())) {
            return invalidValue(l("Date"), val, key, parent);
        }
        return d;
    }

    function transformObject(props: { [k: string]: any }, additional: any, val: any): any {
        if (val === null || typeof val !== "object" || Array.isArray(val)) {
            return invalidValue(l(ref || "object"), val, key, parent);
        }
        const result: any = {};
        Object.getOwnPropertyNames(props).forEach(key => {
            const prop = props[key];
            const v = Object.prototype.hasOwnProperty.call(val, key) ? val[key] : undefined;
            result[prop.key] = transform(v, prop.typ, getProps, key, ref);
        });
        Object.getOwnPropertyNames(val).forEach(key => {
            if (!Object.prototype.hasOwnProperty.call(props, key)) {
                result[key] = transform(val[key], additional, getProps, key, ref);
            }
        });
        return result;
    }

    if (typ === "any") return val;
    if (typ === null) {
        if (val === null) return val;
        return invalidValue(typ, val, key, parent);
    }
    if (typ === false) return invalidValue(typ, val, key, parent);
    let ref: any = undefined;
    while (typeof typ === "object" && typ.ref !== undefined) {
        ref = typ.ref;
        typ = typeMap[typ.ref];
    }
    if (Array.isArray(typ)) return transformEnum(typ, val);
    if (typeof typ === "object") {
        return typ.hasOwnProperty("unionMembers") ? transformUnion(typ.unionMembers, val)
            : typ.hasOwnProperty("arrayItems")    ? transformArray(typ.arrayItems, val)
            : typ.hasOwnProperty("props")         ? transformObject(getProps(typ), typ.additional, val)
            : invalidValue(typ, val, key, parent);
    }
    // Numbers can be parsed by Date but shouldn't be.
    if (typ === Date && typeof val !== "number") return transformDate(val);
    return transformPrimitive(typ, val);
}

function cast<T>(val: any, typ: any): T {
    return transform(val, typ, jsonToJSProps);
}

function uncast<T>(val: T, typ: any): any {
    return transform(val, typ, jsToJSONProps);
}

function l(typ: any) {
    return { literal: typ };
}

function a(typ: any) {
    return { arrayItems: typ };
}

function u(...typs: any[]) {
    return { unionMembers: typs };
}

function o(props: any[], additional: any) {
    return { props, additional };
}

function m(additional: any) {
    return { props: [], additional };
}

function r(name: string) {
    return { ref: name };
}

const typeMap: any = {
    "DeploymentSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "$ref", js: "$ref", typ: "" },
        { json: "definitions", js: "definitions", typ: r("DeploymentSchemaDefinitions") },
    ], false),
    "DeploymentSchemaDefinitions": o([
        { json: "DeploymentSchema", js: "DeploymentSchema", typ: r("Schema") },
        { json: "Definitions", js: "Definitions", typ: r("Definitions") },
        { json: "Properties", js: "Properties", typ: r("PurpleProperties") },
        { json: "IP", js: "IP", typ: r("AddVaryHeaderClass") },
    ], false),
    "Definitions": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "title", js: "title", typ: "" },
    ], false),
    "Schema": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("DeploymentSchemaProperties") },
        { json: "required", js: "required", typ: a(r("Required")) },
        { json: "title", js: "title", typ: "" },
    ], false),
    "DeploymentSchemaProperties": o([
        { json: "$schema", js: "$schema", typ: r("SchemaClass") },
        { json: "definitions", js: "definitions", typ: r("IPClass") },
        { json: "properties", js: "properties", typ: r("IPClass") },
        { json: "title", js: "title", typ: r("NameClass") },
        { json: "type", js: "type", typ: r("NameClass") },
    ], false),
    "SchemaClass": o([
        { json: "type", js: "type", typ: r("Type") },
        { json: "format", js: "format", typ: "" },
        { json: "qt-uri-protocols", js: "qt-uri-protocols", typ: a("") },
    ], false),
    "IPClass": o([
        { json: "$ref", js: "$ref", typ: "" },
    ], false),
    "NameClass": o([
        { json: "type", js: "type", typ: r("Type") },
    ], false),
    "AddVaryHeaderClass": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("IPProperties") },
        { json: "required", js: "required", typ: a(r("Required")) },
        { json: "title", js: "title", typ: "" },
    ], false),
    "IPProperties": o([
        { json: "type", js: "type", typ: r("NameClass") },
    ], false),
    "PurpleProperties": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("FluffyProperties") },
        { json: "required", js: "required", typ: a("") },
        { json: "title", js: "title", typ: "" },
    ], false),
    "FluffyProperties": o([
        { json: "ip", js: "ip", typ: r("IPClass") },
        { json: "name", js: "name", typ: r("IPClass") },
        { json: "port", js: "port", typ: r("IPClass") },
        { json: "protocol", js: "protocol", typ: r("NameClass") },
        { json: "weight", js: "weight", typ: r("IPClass") },
    ], false),
    "FromClientIPSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "$ref", js: "$ref", typ: "" },
        { json: "definitions", js: "definitions", typ: r("FromClientIPSchemaDefinitions") },
    ], false),
    "FromClientIPSchemaDefinitions": o([
        { json: "FromClientIPSchema", js: "FromClientIPSchema", typ: r("Schema") },
        { json: "Definitions", js: "Definitions", typ: r("Definitions") },
        { json: "Properties", js: "Properties", typ: r("TentacledProperties") },
    ], false),
    "TentacledProperties": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("StickyProperties") },
        { json: "required", js: "required", typ: a("") },
        { json: "title", js: "title", typ: "" },
    ], false),
    "StickyProperties": o([
        { json: "ip", js: "ip", typ: r("NameClass") },
        { json: "range", js: "range", typ: r("NameClass") },
    ], false),
    "HeadersSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "$ref", js: "$ref", typ: "" },
        { json: "definitions", js: "definitions", typ: r("HeadersSchemaDefinitions") },
    ], false),
    "HeadersSchemaDefinitions": o([
        { json: "HeadersSchema", js: "HeadersSchema", typ: r("Schema") },
        { json: "Definitions", js: "Definitions", typ: r("Definitions") },
        { json: "Properties", js: "Properties", typ: r("IndigoProperties") },
        { json: "AccessControl", js: "AccessControl", typ: r("AccessControl") },
        { json: "AddVaryHeader", js: "AddVaryHeader", typ: r("AddVaryHeaderClass") },
        { json: "CustomReHeaders", js: "CustomReHeaders", typ: r("CustomReHeaders") },
    ], false),
    "AccessControl": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("AccessControlProperties") },
        { json: "required", js: "required", typ: a("") },
        { json: "title", js: "title", typ: "" },
    ], false),
    "AccessControlProperties": o([
        { json: "items", js: "items", typ: r("NameClass") },
        { json: "type", js: "type", typ: r("NameClass") },
    ], false),
    "CustomReHeaders": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("CustomReHeadersProperties") },
        { json: "required", js: "required", typ: a("") },
        { json: "title", js: "title", typ: "" },
    ], false),
    "CustomReHeadersProperties": o([
        { json: "additionalProperties", js: "additionalProperties", typ: r("NameClass") },
        { json: "type", js: "type", typ: r("NameClass") },
    ], false),
    "IndigoProperties": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("IndecentProperties") },
        { json: "required", js: "required", typ: a("") },
        { json: "title", js: "title", typ: "" },
    ], false),
    "IndecentProperties": o([
        { json: "accessControlAllowHeaders", js: "accessControlAllowHeaders", typ: r("IPClass") },
        { json: "accessControlAllowMethods", js: "accessControlAllowMethods", typ: r("IPClass") },
        { json: "accessControlAllowOriginList", js: "accessControlAllowOriginList", typ: r("IPClass") },
        { json: "accessControlExposeHeaders", js: "accessControlExposeHeaders", typ: r("IPClass") },
        { json: "addVaryHeader", js: "addVaryHeader", typ: r("IPClass") },
        { json: "customRequestHeaders", js: "customRequestHeaders", typ: r("IPClass") },
        { json: "customResponseHeaders", js: "customResponseHeaders", typ: r("IPClass") },
    ], false),
    "HostSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "$ref", js: "$ref", typ: "" },
        { json: "definitions", js: "definitions", typ: r("HostSchemaDefinitions") },
    ], false),
    "HostSchemaDefinitions": o([
        { json: "HostSchema", js: "HostSchema", typ: r("Schema") },
        { json: "Definitions", js: "Definitions", typ: r("Definitions") },
        { json: "Properties", js: "Properties", typ: r("HilariousProperties") },
        { json: "Deployments", js: "Deployments", typ: r("CustomReHeaders") },
        { json: "Domain", js: "Domain", typ: r("AddVaryHeaderClass") },
        { json: "Middlewares", js: "Middlewares", typ: r("AccessControl") },
    ], false),
    "HilariousProperties": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("AmbitiousProperties") },
        { json: "required", js: "required", typ: a("") },
        { json: "title", js: "title", typ: "" },
    ], false),
    "AmbitiousProperties": o([
        { json: "deployments", js: "deployments", typ: r("IPClass") },
        { json: "domain", js: "domain", typ: r("IPClass") },
        { json: "middlewares", js: "middlewares", typ: r("IPClass") },
        { json: "paths", js: "paths", typ: r("IPClass") },
    ], false),
    "MiddlewareSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "$ref", js: "$ref", typ: "" },
        { json: "definitions", js: "definitions", typ: r("MiddlewareSchemaDefinitions") },
    ], false),
    "MiddlewareSchemaDefinitions": o([
        { json: "MiddlewareSchema", js: "MiddlewareSchema", typ: r("Schema") },
        { json: "Definitions", js: "Definitions", typ: r("Definitions") },
        { json: "Properties", js: "Properties", typ: r("CunningProperties") },
        { json: "Name", js: "Name", typ: r("AddVaryHeaderClass") },
    ], false),
    "CunningProperties": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("MagentaProperties") },
        { json: "required", js: "required", typ: a("") },
        { json: "title", js: "title", typ: "" },
    ], false),
    "MagentaProperties": o([
        { json: "name", js: "name", typ: r("IPClass") },
        { json: "protocol", js: "protocol", typ: r("IPClass") },
    ], false),
    "PathSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "$ref", js: "$ref", typ: "" },
        { json: "definitions", js: "definitions", typ: r("PathSchemaDefinitions") },
    ], false),
    "PathSchemaDefinitions": o([
        { json: "PathSchema", js: "PathSchema", typ: r("Schema") },
        { json: "Definitions", js: "Definitions", typ: r("Definitions") },
        { json: "Properties", js: "Properties", typ: r("FriskyProperties") },
        { json: "Deployments", js: "Deployments", typ: r("CustomReHeaders") },
        { json: "Middlewares", js: "Middlewares", typ: r("AccessControl") },
        { json: "PassThrough", js: "PassThrough", typ: r("AddVaryHeaderClass") },
    ], false),
    "FriskyProperties": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("MischievousProperties") },
        { json: "required", js: "required", typ: a("") },
        { json: "title", js: "title", typ: "" },
    ], false),
    "MischievousProperties": o([
        { json: "deployments", js: "deployments", typ: r("IPClass") },
        { json: "middlewares", js: "middlewares", typ: r("IPClass") },
        { json: "passThrough", js: "passThrough", typ: r("IPClass") },
        { json: "path", js: "path", typ: r("IPClass") },
        { json: "stripPrefix", js: "stripPrefix", typ: r("IPClass") },
    ], false),
    "SelectionSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "$ref", js: "$ref", typ: "" },
        { json: "definitions", js: "definitions", typ: r("SelectionSchemaDefinitions") },
    ], false),
    "SelectionSchemaDefinitions": o([
        { json: "SelectionSchema", js: "SelectionSchema", typ: r("Schema") },
        { json: "Definitions", js: "Definitions", typ: r("Definitions") },
    ], false),
    "TraefikSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "$ref", js: "$ref", typ: "" },
        { json: "definitions", js: "definitions", typ: r("TraefikSchemaDefinitions") },
    ], false),
    "TraefikSchemaDefinitions": o([
        { json: "TraefikSchema", js: "TraefikSchema", typ: r("Schema") },
        { json: "Definitions", js: "Definitions", typ: r("Definitions") },
        { json: "TraefikSchemaProperties", js: "TraefikSchemaProperties", typ: r("TraefikSchemaProperties") },
        { json: "Etcd", js: "Etcd", typ: r("Etcd") },
        { json: "EtcdProperties", js: "EtcdProperties", typ: r("DefinitionsEtcdProperties") },
        { json: "Endpoints", js: "Endpoints", typ: r("Endpoints") },
        { json: "RulePrefix", js: "RulePrefix", typ: r("AddVaryHeaderClass") },
        { json: "Hosts", js: "Hosts", typ: r("AccessControl") },
        { json: "Middlewares", js: "Middlewares", typ: r("CustomReHeaders") },
    ], false),
    "Endpoints": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("EndpointsProperties") },
        { json: "required", js: "required", typ: a("") },
        { json: "title", js: "title", typ: "" },
    ], false),
    "EndpointsProperties": o([
        { json: "items", js: "items", typ: r("IPClass") },
        { json: "type", js: "type", typ: r("NameClass") },
    ], false),
    "Etcd": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("EtcdProperties") },
        { json: "required", js: "required", typ: a(r("Required")) },
        { json: "title", js: "title", typ: "" },
    ], false),
    "EtcdProperties": o([
        { json: "properties", js: "properties", typ: r("IPClass") },
        { json: "type", js: "type", typ: r("NameClass") },
    ], false),
    "DefinitionsEtcdProperties": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("EtcdPropertiesProperties") },
        { json: "required", js: "required", typ: a("") },
        { json: "title", js: "title", typ: "" },
    ], false),
    "EtcdPropertiesProperties": o([
        { json: "endpoints", js: "endpoints", typ: r("IPClass") },
        { json: "keep_alive", js: "keep_alive", typ: r("IPClass") },
        { json: "timeout", js: "timeout", typ: r("IPClass") },
    ], false),
    "TraefikSchemaProperties": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("TraefikSchemaPropertiesProperties") },
        { json: "required", js: "required", typ: a("") },
        { json: "title", js: "title", typ: "" },
    ], false),
    "TraefikSchemaPropertiesProperties": o([
        { json: "etcd", js: "etcd", typ: r("IPClass") },
        { json: "hosts", js: "hosts", typ: r("IPClass") },
        { json: "middlewares", js: "middlewares", typ: r("IPClass") },
        { json: "rulePrefix", js: "rulePrefix", typ: r("IPClass") },
    ], false),
    "TraefikctlConfigDeploymentDeploymentConfigSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "definitions", js: "definitions", typ: r("TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions") },
        { json: "properties", js: "properties", typ: r("TraefikctlConfigDeploymentDeploymentConfigSchemaProperties") },
        { json: "title", js: "title", typ: "" },
        { json: "type", js: "type", typ: "" },
    ], false),
    "TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions": o([
    ], false),
    "TraefikctlConfigDeploymentDeploymentConfigSchemaProperties": o([
        { json: "ip", js: "ip", typ: r("NameClass") },
        { json: "name", js: "name", typ: r("NameClass") },
        { json: "port", js: "port", typ: r("NameClass") },
        { json: "protocol", js: "protocol", typ: true },
        { json: "weight", js: "weight", typ: r("NameClass") },
    ], false),
    "TraefikctlConfigHeadersHeadersConfigSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "definitions", js: "definitions", typ: r("TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions") },
        { json: "properties", js: "properties", typ: r("TraefikctlConfigHeadersHeadersConfigSchemaProperties") },
        { json: "title", js: "title", typ: "" },
        { json: "type", js: "type", typ: "" },
    ], false),
    "TraefikctlConfigHeadersHeadersConfigSchemaProperties": o([
        { json: "accessControlAllowHeaders", js: "accessControlAllowHeaders", typ: r("AccessControlAllowHeaders") },
        { json: "accessControlAllowMethods", js: "accessControlAllowMethods", typ: r("AccessControlAllowHeaders") },
        { json: "accessControlAllowOriginList", js: "accessControlAllowOriginList", typ: r("AccessControlAllowHeaders") },
        { json: "accessControlExposeHeaders", js: "accessControlExposeHeaders", typ: r("AccessControlAllowHeaders") },
        { json: "addVaryHeader", js: "addVaryHeader", typ: r("NameClass") },
        { json: "customRequestHeaders", js: "customRequestHeaders", typ: r("CustomRequestHeaders") },
        { json: "customResponseHeaders", js: "customResponseHeaders", typ: r("CustomRequestHeaders") },
    ], false),
    "AccessControlAllowHeaders": o([
        { json: "items", js: "items", typ: true },
        { json: "type", js: "type", typ: "" },
    ], false),
    "CustomRequestHeaders": o([
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "type", js: "type", typ: "" },
    ], false),
    "TraefikctlConfigHostHostConfigSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "definitions", js: "definitions", typ: r("TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions") },
        { json: "properties", js: "properties", typ: r("TraefikctlConfigHostHostConfigSchemaProperties") },
        { json: "title", js: "title", typ: "" },
        { json: "type", js: "type", typ: "" },
    ], false),
    "TraefikctlConfigHostHostConfigSchemaProperties": o([
        { json: "deployments", js: "deployments", typ: r("CustomRequestHeaders") },
        { json: "domain", js: "domain", typ: r("NameClass") },
        { json: "middlewares", js: "middlewares", typ: r("AccessControlAllowHeaders") },
        { json: "paths", js: "paths", typ: r("AccessControlAllowHeaders") },
    ], false),
    "TraefikctlConfigHostPathConfigSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "definitions", js: "definitions", typ: r("TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions") },
        { json: "properties", js: "properties", typ: r("TraefikctlConfigHostPathConfigSchemaProperties") },
        { json: "title", js: "title", typ: "" },
        { json: "type", js: "type", typ: "" },
    ], false),
    "TraefikctlConfigHostPathConfigSchemaProperties": o([
        { json: "deployments", js: "deployments", typ: r("CustomRequestHeaders") },
        { json: "middlewares", js: "middlewares", typ: r("AccessControlAllowHeaders") },
        { json: "passThrough", js: "passThrough", typ: r("NameClass") },
        { json: "path", js: "path", typ: r("NameClass") },
        { json: "stripPrefix", js: "stripPrefix", typ: r("NameClass") },
    ], false),
    "TraefikctlConfigMiddlewareMiddlewareConfigSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "definitions", js: "definitions", typ: r("TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions") },
        { json: "properties", js: "properties", typ: r("TraefikctlConfigMiddlewareMiddlewareConfigSchemaProperties") },
        { json: "title", js: "title", typ: "" },
        { json: "type", js: "type", typ: "" },
    ], false),
    "TraefikctlConfigMiddlewareMiddlewareConfigSchemaProperties": o([
        { json: "name", js: "name", typ: r("NameClass") },
        { json: "protocol", js: "protocol", typ: r("NameClass") },
    ], false),
    "TraefikctlConfigSelectionsFromClientIPConfigSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "definitions", js: "definitions", typ: r("TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions") },
        { json: "properties", js: "properties", typ: r("TraefikctlConfigSelectionsFromClientIPConfigSchemaProperties") },
        { json: "title", js: "title", typ: "" },
        { json: "type", js: "type", typ: "" },
    ], false),
    "TraefikctlConfigSelectionsFromClientIPConfigSchemaProperties": o([
        { json: "ip", js: "ip", typ: true },
        { json: "range", js: "range", typ: true },
    ], false),
    "TraefikctlConfigSelectionsSelectionConfigSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "definitions", js: "definitions", typ: r("TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions") },
        { json: "properties", js: "properties", typ: r("TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions") },
        { json: "title", js: "title", typ: "" },
        { json: "type", js: "type", typ: "" },
    ], false),
    "TraefikctlConfigSelectionsWithCookieConfigSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "definitions", js: "definitions", typ: r("TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions") },
        { json: "properties", js: "properties", typ: r("TraefikctlConfigSelectionsWithCookieConfigSchemaProperties") },
        { json: "title", js: "title", typ: "" },
        { json: "type", js: "type", typ: "" },
    ], false),
    "TraefikctlConfigSelectionsWithCookieConfigSchemaProperties": o([
        { json: "name", js: "name", typ: r("NameClass") },
        { json: "value", js: "value", typ: true },
    ], false),
    "TraefikctlConfigTraefikConfigTraefikConfigSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "definitions", js: "definitions", typ: r("TraefikctlConfigDeploymentDeploymentConfigSchemaDefinitions") },
        { json: "properties", js: "properties", typ: r("TraefikctlConfigTraefikConfigTraefikConfigSchemaProperties") },
        { json: "title", js: "title", typ: "" },
        { json: "type", js: "type", typ: "" },
    ], false),
    "TraefikctlConfigTraefikConfigTraefikConfigSchemaProperties": o([
        { json: "etcd", js: "etcd", typ: r("EtcdClass") },
        { json: "hosts", js: "hosts", typ: r("AccessControlAllowHeaders") },
        { json: "middlewares", js: "middlewares", typ: r("CustomRequestHeaders") },
        { json: "rulePrefix", js: "rulePrefix", typ: r("NameClass") },
    ], false),
    "EtcdClass": o([
        { json: "properties", js: "properties", typ: r("EtcdPropertiesClass") },
        { json: "type", js: "type", typ: "" },
    ], false),
    "EtcdPropertiesClass": o([
        { json: "endpoints", js: "endpoints", typ: r("EndpointsClass") },
        { json: "keep_alive", js: "keep_alive", typ: r("NameClass") },
        { json: "timeout", js: "timeout", typ: r("NameClass") },
    ], false),
    "EndpointsClass": o([
        { json: "items", js: "items", typ: r("NameClass") },
        { json: "type", js: "type", typ: "" },
    ], false),
    "WithCookieSchema": o([
        { json: "$schema", js: "$schema", typ: "" },
        { json: "$ref", js: "$ref", typ: "" },
        { json: "definitions", js: "definitions", typ: r("WithCookieSchemaDefinitions") },
    ], false),
    "WithCookieSchemaDefinitions": o([
        { json: "WithCookieSchema", js: "WithCookieSchema", typ: r("Schema") },
        { json: "Definitions", js: "Definitions", typ: r("Definitions") },
        { json: "Properties", js: "Properties", typ: r("BraggadociousProperties") },
        { json: "Name", js: "Name", typ: r("AddVaryHeaderClass") },
    ], false),
    "BraggadociousProperties": o([
        { json: "type", js: "type", typ: "" },
        { json: "additionalProperties", js: "additionalProperties", typ: true },
        { json: "properties", js: "properties", typ: r("Properties1") },
        { json: "required", js: "required", typ: a("") },
        { json: "title", js: "title", typ: "" },
    ], false),
    "Properties1": o([
        { json: "name", js: "name", typ: r("IPClass") },
        { json: "value", js: "value", typ: r("NameClass") },
    ], false),
    "Type": [
        "boolean",
        "integer",
        "string",
    ],
    "Required": [
        "definitions",
        "properties",
        "$schema",
        "title",
        "type",
    ],
};
