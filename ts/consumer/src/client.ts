import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { Secp256k1KeyIdentity } from "@dfinity/identity-secp256k1";
import pemfile from "pem-file";
import { EventSinkCanister, idlFactory } from "./candid/idl";
import type { EventsResponse, RemoveEventsResponse } from "./types";
import { candidEventsResponse, candidRemoveEventsResponse } from "./mappers";

export class Client {
  private readonly canister: EventSinkCanister;

  public constructor(canisterId: string | Principal, agent: HttpAgent) {
    this.canister = Actor.createActor(idlFactory, {
      agent,
      canisterId,
    });
  }

  public static createFromPem(canisterId: string | Principal, pem: string): Client {
    const buf = pemfile.decode(pem);
    if (buf.length != 118) {
      throw 'expecting byte length 118 but got ' + buf.length;
    }
    const identity = Secp256k1KeyIdentity.fromSecretKey(buf.subarray(7, 39));
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

  public async removeEvents(
    upToInclusive: bigint,
  ): Promise<RemoveEventsResponse> {
    const candid = await this.canister.remove_events({
      up_to_inclusive: upToInclusive,
    });
    return candidRemoveEventsResponse(candid);
  }
}
