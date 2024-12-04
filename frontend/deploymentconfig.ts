export interface DeploymentConfig {
    name: string;
    ip: string;
    port: number;
    weight: usize;
    selection?: SelectionConfig | undefined;
    protocol: DeploymentProtocol;
    middlewares?: string[] | undefined;
}