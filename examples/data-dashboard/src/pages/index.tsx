import React, { useEffect, useState } from 'react';

interface Kpis {
    totalRevenue: number;
    totalUnits: number;
    totalOrders: number;
    avgOrderValue: number;
    growthPercent: number;
    bestMonth: string;
    topCategory: string;
}

interface MonthlyPoint {
    month: string;
    revenue: number;
}

interface ShareRow {
    category?: string;
    region?: string;
    revenue: number;
    share: number;
}

interface ProductRow {
    product: string;
    category: string;
    revenue: number;
    units: number;
}

interface Analysis {
    dataset: { records: number; products: string[]; categories: string[]; regions: string[]; months: string[] };
    summary: Kpis;
    monthlyRevenue: MonthlyPoint[];
    categoryRevenue: ShareRow[];
    regionRevenue: ShareRow[];
    topProducts: ProductRow[];
}

interface OgResult {
    page: string;
    width: number;
    height: number;
    contentType: string;
    image: string;
    generatedAt: string;
}

function fmt(n: number): string {
    return n.toLocaleString('en-US');
}

function money(n: number): string {
    return '$' + n.toLocaleString('en-US');
}

function maxOf(points: MonthlyPoint[]): number {
    return Math.max(...points.map((p) => p.revenue));
}

export default function Index() {
    const [analysis, setAnalysis] = useState<Analysis | null>(null);
    const [og, setOg] = useState<OgResult | null>(null);
    const [error, setError] = useState<string | null>(null);

    async function loadOg() {
        const res = await fetch('/api/og?page=dashboard&t=' + Date.now());
        const json = await res.json();
        setOg(json);
    }

    useEffect(() => {
        fetch('/api/analysis')
            .then((r) => r.json())
            .then((json) => setAnalysis(json))
            .catch(() => setError('Could not reach the NumPy analysis endpoint.'));

        loadOg().catch(() => setError('Could not reach the Node OG-image endpoint.'));
    }, []);

    const kpi = analysis?.summary;
    const maxMonthly = analysis ? maxOf(analysis.monthlyRevenue) : 0;
    const ogSrc = og ? `data:image/png;base64,${og.image}` : null;

    return (
        <>
            <section className="section">
                <p className="eyebrow">Live &middot; NumPy backend</p>
                <h2 className="sectionTitle">Key numbers</h2>
                {error ? (
                    <p className="sectionLead">{error}</p>
                ) : !kpi ? (
                    <p className="sectionLead">Loading analysis from /api/analysis&hellip;</p>
                ) : (
                    <div className="kpiGrid">
                        <div className="kpi">
                            <span className="kpiLabel">Total revenue</span>
                            <span className="kpiValue">{money(kpi.totalRevenue)}</span>
                        </div>
                        <div className="kpi">
                            <span className="kpiLabel">Average order</span>
                            <span className="kpiValue">{money(kpi.avgOrderValue)}</span>
                        </div>
                        <div className="kpi">
                            <span className="kpiLabel">Units sold</span>
                            <span className="kpiValue">{fmt(kpi.totalUnits)}</span>
                        </div>
                        <div className="kpi">
                            <span className="kpiLabel">Half-year growth</span>
                            <span className="kpiValue">{kpi.growthPercent}%</span>
                        </div>
                        <div className="kpi">
                            <span className="kpiLabel">Top category</span>
                            <span className="kpiValue">{kpi.topCategory}</span>
                        </div>
                        <div className="kpi">
                            <span className="kpiLabel">Best month</span>
                            <span className="kpiValue">{kpi.bestMonth}</span>
                        </div>
                    </div>
                )}
            </section>

            <section className="section">
                <p className="eyebrow">Series &middot; NumPy backend</p>
                <h2 className="sectionTitle">Monthly revenue</h2>
                {analysis && (
                    <div className="chart" aria-label="Monthly revenue bar chart">
                        {analysis.monthlyRevenue.map((point) => (
                            <div className="chartColumn" key={point.month}>
                                <div
                                    className="chartBar"
                                    style={{ height: `${Math.round((point.revenue / maxMonthly) * 100)}%` }}
                                    title={`${point.month}: ${money(point.revenue)}`}
                                />
                                <span className="chartLabel">{point.month}</span>
                            </div>
                        ))}
                    </div>
                )}
            </section>

            <section className="section split">
                <div>
                    <p className="eyebrow">Breakdown &middot; NumPy backend</p>
                    <h2 className="sectionTitle">By category</h2>
                    {analysis && (
                        <ul className="shareList">
                            {analysis.categoryRevenue.map((row) => (
                                <li key={row.category}>
                                    <span>{row.category}</span>
                                    <span className="shareBar">
                                        <span className="shareFill" style={{ width: `${row.share}%` }} />
                                    </span>
                                    <span className="sharePct isAccent">{row.share}%</span>
                                </li>
                            ))}
                        </ul>
                    )}
                </div>
                <div>
                    <p className="eyebrow">Breakdown &middot; NumPy backend</p>
                    <h2 className="sectionTitle">By region</h2>
                    {analysis && (
                        <ul className="shareList">
                            {analysis.regionRevenue.map((row) => (
                                <li key={row.region}>
                                    <span>{row.region}</span>
                                    <span className="shareBar">
                                        <span className="shareFill" style={{ width: `${row.share}%` }} />
                                    </span>
                                    <span className="sharePct isAccent">{row.share}%</span>
                                </li>
                            ))}
                        </ul>
                    )}
                </div>
            </section>

            <section className="section">
                <p className="eyebrow">Ranking &middot; NumPy backend</p>
                <h2 className="sectionTitle">Top products</h2>
                {analysis && (
                    <div className="tableWrap">
                        <table className="dataTable">
                            <thead>
                                <tr>
                                    <th>Product</th>
                                    <th>Category</th>
                                    <th>Revenue</th>
                                    <th>Units</th>
                                </tr>
                            </thead>
                            <tbody>
                                {analysis.topProducts.map((row) => (
                                    <tr key={row.product}>
                                        <th>{row.product}</th>
                                        <td>{row.category}</td>
                                        <td className="isAccent">{money(row.revenue)}</td>
                                        <td>{fmt(row.units)}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </section>

            <section className="section">
                <p className="eyebrow">Open Graph &middot; Node backend</p>
                <h2 className="sectionTitle">Link-preview image</h2>
                <p className="sectionLead">
                    Generated by /api/og in the Node loader: a 1200&times;630 PNG drawn
                    pixel by pixel, no image library, no external service.
                </p>
                {!ogSrc ? (
                    <p className="sectionLead">Rendering with Node&hellip;</p>
                ) : (
                    <div className="ogCard">
                        <img src={ogSrc} alt="Open Graph preview generated by Node" width={1200} height={630} />
                        <div className="ogMeta">
                            <span className="pill">{og.page}</span>
                            <span className="cta">
                                {og.width}&times;{og.height} &middot; {og.contentType}
                            </span>
                            <button className="button" onClick={() => loadOg().catch(() => setError('OG re-render failed.'))}>
                                Re-render
                            </button>
                            <a className="button" href={ogSrc} download={`og-${og.page}.png`}>
                                Download
                            </a>
                        </div>
                    </div>
                )}
            </section>
        </>
    );
}