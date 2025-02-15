/**
 * Group elements into two based on a binary predicate
 * @param arr
 * @param predicate
 * @returns [is true, is false]
 */
export const partition = <T>(arr: T[], predicate: (v: T, i: number, ar: T[]) => boolean) =>
  arr.reduce(
    (acc, item, index, array) => {
      acc[+!predicate(item, index, array)].push(item)
      return acc
    },
    [[], []] as [T[], T[]]
  )

export function inUnion<T extends readonly string[]>(union: T, val: string): val is T[number] {
  return union.includes(val)
}

export function invariantUnion<T extends readonly string[]>(union: T, val: string): asserts val is T[number] {
  if (!union.includes(val)) {
    throw new Error(val + ' is not part of the union ' + union.toString())
  }
}

export function assertUnion<T extends readonly string[]>(union: T, val: string): T[number] {
  if (!union.includes(val)) {
    throw new Error(val + ' is not part of the union ' + union.toString())
  }
  return val
}
