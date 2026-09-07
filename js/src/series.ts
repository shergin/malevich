export type Gap = null | undefined;
export type SeriesLike = ArrayLike<number | Gap> | Iterable<number | Gap>;

export function toFloat64(values: SeriesLike): Float64Array {
  if (values instanceof Float64Array) {
    return values;
  }
  if (ArrayBuffer.isView(values) || Array.isArray(values)) {
    const list = values as ArrayLike<number | Gap>;
    const out = new Float64Array(list.length);
    for (let i = 0; i < list.length; i++) {
      out[i] = toScalar(list[i]);
    }
    return out;
  }
  if (typeof values === "object" && values !== null && "length" in values) {
    const list = values as ArrayLike<number | Gap>;
    const out = new Float64Array(list.length);
    for (let i = 0; i < list.length; i++) {
      out[i] = toScalar(list[i]);
    }
    return out;
  }
  const collected: number[] = [];
  for (const value of values) {
    collected.push(toScalar(value));
  }
  return Float64Array.from(collected);
}

function toScalar(value: number | Gap): number {
  if (value === null || value === undefined) {
    return Number.NaN;
  }
  return typeof value === "number" ? value : Number(value);
}

export type ColRef = { col: number };

export function col(index: number): ColRef {
  return { col: index };
}
