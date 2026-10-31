export type Benchmark = {
	metric: string;
	metassr: string;
	nextjs: string;
	gain: string;
	info?: string;
};

export const apiBenchmarks: Benchmark[] = [
	{ metric: 'Requests/sec', metassr: '107,116.71', nextjs: '1,469.79', gain: '73x faster' },
	{ metric: 'Average Latency', metassr: '10.15ms', nextjs: '126.75ms', gain: '12x lower' },
	{ metric: 'Transfer/sec', metassr: '16.96MB', nextjs: '457.89KB', gain: '38x higher' },
	{ metric: 'Total Requests', metassr: '3,216,110', nextjs: '44,239', gain: '73x more' },
	{ metric: 'Max Latency', metassr: '71.29ms', nextjs: '1.97s', gain: '28x lower' },
	{
		metric: 'Socket Errors',
		metassr: '0',
		nextjs: '237 timeouts',
		gain: 'Zero errors',
		info: 'Socket errors are client-side wrk connection failures: connect, read, write, or timeout (2s default). They are not HTTP errors. 0 means every request completed; timeouts mean the server was saturated and wrk gave up.',
	},
];

export const ssrBenchmarks: Benchmark[] = [
	{ metric: 'Requests/sec', metassr: '41,499.66', nextjs: '500.86', gain: '83x faster' },
	{ metric: 'Average Latency', metassr: '25.36ms', nextjs: '495.70ms', gain: '20x lower' },
	{ metric: 'Transfer/sec', metassr: '120.16MB', nextjs: '5.72MB', gain: '21x higher' },
	{ metric: 'Total Requests', metassr: '1,249,104', nextjs: '15,057', gain: '83x more' },
	{ metric: 'Max Latency', metassr: '158.38ms', nextjs: '1.86s', gain: '12x lower' },
	{
		metric: 'Socket Errors',
		metassr: '0',
		nextjs: '234 timeouts',
		gain: 'Zero errors',
		info: 'Socket errors are client-side wrk connection failures: connect, read, write, or timeout (2s default). They are not HTTP errors. 0 means every request completed; timeouts mean the server was saturated and wrk gave up.',
	},
];
