import "@testing-library/jest-dom/vitest";
import { afterEach } from "vitest";
import { cleanup } from "@testing-library/react";
import { browserMock } from "../src/bridge";
import { DEFAULT_SNAPSHOT } from "../src/types";

afterEach(() => {
  cleanup();
  browserMock.reset(DEFAULT_SNAPSHOT);
  window.history.replaceState({}, "", "/");
});
