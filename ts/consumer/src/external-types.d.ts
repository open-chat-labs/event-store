declare module "pem-file" {
    export function decode(pem: string | Buffer): Buffer;
}
