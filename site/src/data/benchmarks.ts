export type Benchmark = {
	metric: string;
	metassr: string;
	nextjs: string;
	gain: string;
};

export const benchmarks: Benchmark[] = [
	{ metric: 'Requests/sec', metassr: '107,116.71', nextjs: '1,469.79', gain: '73x faster' },
	{ metric: 'Average Latency', metassr: '10.15ms', nextjs: '126.75ms', gain: '12x lower' },
	{ metric: 'Transfer/sec', metassr: '16.96MB', nextjs: '457.89KB', gain: '38x higher' },
	{ metric: 'Total Requests', metassr: '3,216,110', nextjs: '44,239', gain: '73x more' },
	{ metric: 'Max Latency', metassr: '71.29ms', nextjs: '1.97s', gain: '28x lower' },
	{ metric: 'Socket Errors', metassr: '0', nextjs: '237 timeouts', gain: 'Zero errors' },
];
