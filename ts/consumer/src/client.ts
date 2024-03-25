import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { Secp256k1KeyIdentity } from "@dfinity/identity-secp256k1";
import { EventStoreCanister, idlFactory } from "./candid/idl";
import type { EventsResponse } from "./types";
import { candidEventsResponse } from "./mappers";

export class Client {
  private readonly canister: EventStoreCanister;

  public constructor(canisterId: string | Principal, agent: HttpAgent) {
    this.canister = Actor.createActor(idlFactory, {
      agent,
      canisterId,
    });
  }

  public static createFromPem(canisterId: string | Principal, pem: string): Client {
    const identity = Secp256k1KeyIdentity.fromPem(pem);
    const httpAgent = new HttpAgent({ identity });
    return new Client(canisterId, httpAgent);
  }

  public async events(start: bigint, length: bigint): Promise<EventsResponse> {
    const candid = await this.canister.events({
      start,
      length,
    });
    return candidEventsResponse(candid);
  }
}
