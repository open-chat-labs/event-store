import { Principal } from "@dfinity/principal";
import { Actor, HttpAgent } from "@dfinity/agent";
import { EventSinkCanister, idlFactory } from "./candid/idl";
import type { EventsResponse, RemoveEventsResponse } from "./types";
import { candidEventsResponse, candidRemoveEventsResponse } from "./mappers";

export class Client {
  private readonly canister: EventSinkCanister;

  public constructor(canisterId: Principal, agent: HttpAgent) {
    this.canister = Actor.createActor(idlFactory, {
      agent,
      canisterId,
    });
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
