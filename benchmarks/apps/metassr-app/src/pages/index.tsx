import React from "react";

interface Item {
  id: number;
  title: string;
  body: string;
}

function generateItems(count: number): Item[] {
  const items: Item[] = [];
  for (let i = 0; i < count; i++) {
    items.push({
      id: i + 1,
      title: `Item ${i + 1}`,
      body: `This is the body content for item ${i + 1}. It contains some text to simulate real content rendering.`,
    });
  }
  return items;
}

export default function Home() {
  const items = generateItems(20);

  return (
    <main>
      <h1>Benchmark Test Page</h1>
      <p>This page is used for SSR performance benchmarking.</p>
      <ul>
        {items.map((item) => (
          <li key={item.id}>
            <h2>{item.title}</h2>
            <p>{item.body}</p>
          </li>
        ))}
      </ul>
    </main>
  );
}
