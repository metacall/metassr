import { NextRequest, NextResponse } from 'next/server'

export const dynamic = 'force-dynamic'

function payloadSeed(input: string): number {
  let hash = 2166136261
  for (let i = 0; i < input.length; i++) {
    hash ^= input.charCodeAt(i)
    hash = Math.imul(hash, 16777619)
  }
  return hash >>> 0
}

export async function GET(req: NextRequest) {
  const url = new URL(req.url)
  const query: Record<string, string> = {}
  url.searchParams.forEach((value, key) => {
    query[key] = value
  })

  const seed = payloadSeed(
    `${req.method}:${url.pathname}:${JSON.stringify(query)}`
  )

  return NextResponse.json({
    ok: true,
    seed,
    method: req.method,
    queryKeys: Object.keys(query).length,
  })
}

export async function POST(req: NextRequest) {
  const body = await req.json().catch(() => ({}))
  const seed = payloadSeed(JSON.stringify(body))

  return NextResponse.json({
    ok: true,
    seed,
    body,
  })
}
