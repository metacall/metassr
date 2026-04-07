// This file contains invalid JavaScript syntax to test bundling failure
export default function broken() {
  const x = {
    foo: "bar"
    // Missing comma here intentionally
    baz: "qux"
  };
  return x;
}
