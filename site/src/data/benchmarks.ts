export type Benchmark = {
	metric: string;
	metassr: string;
	nextjs: string;
	gain: string;
};

export const benchmarks: Benchmark[] = [
	{ metric: 'Requests/sec', metassr: '98,420.11', nextjs: '3,170.95', gain: '31x faster' },
	{ metric: 'Average Latency', metassr: '8.63ms', nextjs: '119.90ms', gain: '14x lower' },
	{ metric: 'Transfer/sec', metassr: '4.98GB', nextjs: '37.95MB', gain: '134x higher' },
	{ metric: 'Total Requests', metassr: '2,962,418', nextjs: '95,326', gain: '31x more' },
	{ metric: 'Max Latency', metassr: '65.22ms', nextjs: '1.99s', gain: '30x lower' },
	{ metric: 'Socket Errors', metassr: '0', nextjs: '239 timeouts', gain: 'Zero errors' },
];
