export type EventsResponse = {
  events: Event[];
  latestEventIndex: bigint | undefined;
};

export type Event = {
  name: string;
  timestamp: bigint;
  user: string | undefined;
  source: string | undefined;
  payload: Uint8Array;
};
