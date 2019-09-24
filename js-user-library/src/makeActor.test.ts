import * as cbor from "./cbor";

import {
  CanisterId,
  IDL,
  Int,
  makeActor,
  makeCallRequest,
  makeHttpAgent,
  requestIdOf,
} from "./index";

test("makeActor", async () => {
  const actorInterface = new IDL.ActorInterface({
    greet: IDL.Fn([IDL.Text], [IDL.Text]),
  });

  const mockFetch: jest.Mock = jest.fn()
    .mockImplementationOnce((/*resource, init*/) => {
      return Promise.resolve(new Response(null, {
        status: 202,
      }));
    })
    .mockImplementationOnce((resource, init) => {
      const body = cbor.encode({ status: "unknown" });
      return Promise.resolve(new Response(body, {
        status: 200,
      }));
    })
    .mockImplementationOnce((resource, init) => {
      const body = cbor.encode({ status: "pending" });
      return Promise.resolve(new Response(body, {
        status: 200,
      }));
    })
    .mockImplementationOnce((resource, init) => {
      const body = cbor.encode({ status: "replied", reply: "Hello, World!" });
      return Promise.resolve(new Response(body, {
        status: 200,
      }));
    });

  const canisterId = [1] as CanisterId;
  const methodName = "greet";
  const arg: Array<Int> = [];

  const expectedRequest = makeCallRequest({
    canisterId,
    methodName,
    arg,
  });

  const expectedRequestId = await requestIdOf(expectedRequest);

  const httpAgent = makeHttpAgent({
    canisterId,
    fetch: mockFetch,
  });

  const actor = makeActor(actorInterface)(httpAgent);
  // FIXME: the argument isn't actually used yet
  const reply = await actor.greet("Name");

  expect(reply).toBe("Hello, World!");

  const { calls, results } = mockFetch.mock;
  expect(calls.length).toBe(4);

  expect(calls[0][0]).toBe("http://localhost:8080/api/v1/submit");
  expect(calls[0][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    body: cbor.encode(expectedRequest),
  });

  expect(calls[1][0]).toBe("http://localhost:8080/api/v1/read");
  expect(calls[1][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    body: cbor.encode({
      request_type: "request-status",
      request_id: expectedRequestId,
    }),
  });

  expect(calls[2][0]).toBe("http://localhost:8080/api/v1/read");
  expect(calls[2][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    body: cbor.encode({
      request_type: "request-status",
      request_id: expectedRequestId,
    }),
  });

  expect(calls[3][0]).toBe("http://localhost:8080/api/v1/read");
  expect(calls[3][1]).toEqual({
    method: "POST",
    headers: {
      "Content-Type": "application/cbor",
    },
    body: cbor.encode({
      request_type: "request-status",
      request_id: expectedRequestId,
    }),
  });
});

// TODO: tests for rejected, unknown time out
