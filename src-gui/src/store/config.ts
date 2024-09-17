import { ExtendedProviderStatus } from "models/apiModel";

export const isTestnet = () => true;

export const isDevelopment = true;

export function getStubTestnetProvider(): ExtendedProviderStatus | null {
  return {
    multiAddr: "/ip4/127.0.0.1/tcp/9939",
    testnet: true,
    peerId: "12D3KooWEJxNSpzsUiFTL5etJBcfH9GsUJGb2BBUCS9HGVu19idT",
  };
}
