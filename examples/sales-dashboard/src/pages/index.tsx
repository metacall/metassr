import React, { useEffect, useState } from 'react';

type Summary = {
  totalRevenue: number;
  avgOrder: number;
  totalOrders: number;
  totalUnits: number;
};

type MonthlyPoint = {
  month: string;
  revenue: number;
  growthPercent: number;
  rollingAverage: number;
};

type CategoryPoint = {
  category: string;
  revenue: number;
};

type RegionPoint = {
  region: string;
  revenue: number;
};

type ProductPoint = {
  product: string;
  category: string;
  totalRevenue: number;
  totalUnits: number;
  orders: number;
};

type SalesPayload = {
  summary: Summary;
  graphs: {
    categoryRevenue: CategoryPoint[];
    monthlyRevenue: MonthlyPoint[];
    regionRevenue: RegionPoint[];
    topProducts: ProductPoint[];
  };
  filters: {
    regions: string[];
    products: string[];
  };
};

type Filters = {
  region: string;
  dateFrom: string;
  dateTo: string;
};

const DEFAULT_REGIONS = ['North', 'South', 'East', 'West'];

function formatCurrency(value: number) {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    maximumFractionDigits: 0,
  }).format(value);
}

function formatCompactCurrency(value: number) {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    maximumFractionDigits: 1,
    notation: 'compact',
  }).format(value);
}

function formatMonthLabel(month: string) {
  const [year, monthIndex] = month.split('-').map(Number);
  if (!year || !monthIndex) {
    return month;
  }
  return new Date(year, monthIndex - 1, 1).toLocaleString('en-US', { month: 'short' });
}

async function fetchSales(filters?: Filters): Promise<SalesPayload> {
  const hasFilters = !!filters && (filters.region || filters.dateFrom || filters.dateTo);
  const response = await fetch('/api/sales', {
    method: hasFilters ? 'POST' : 'GET',
    headers: hasFilters ? { 'Content-Type': 'application/json' } : undefined,
    body: hasFilters
      ? JSON.stringify({
        region: filters?.region || undefined,
        date_from: filters?.dateFrom || undefined,
        date_to: filters?.dateTo || undefined,
      })
      : undefined,
  });

  const data = await response.json();
  if (!response.ok || (typeof data.status === 'number' && data.status >= 400)) {
    throw new Error(data?.error || 'Failed to load sales data');
  }

  return (data.body || data) as SalesPayload;
}

function MetricCard({ label, value, hint }: { label: string; value: string; hint: string }) {
  return (
    <article className="metricCard">
      <p className="metricLabel">{label}</p>
      <strong className="metricValue">{value}</strong>
      <span className="metricHint">{hint}</span>
    </article>
  );
}

function HorizontalBarChart<T extends Record<string, string | number>>({
  rows,
  labelKey,
  valueKey,
  valueFormatter,
}: {
  rows: T[];
  labelKey: keyof T;
  valueKey: keyof T;
  valueFormatter: (value: number) => string;
}) {
  const maxValue = Math.max(...rows.map(row => Number(row[valueKey])), 1);

  return (
    <div className="barChart">
      {rows.map(row => {
        const value = Number(row[valueKey]);
        const label = String(row[labelKey]);
        const width = `${Math.max((value / maxValue) * 100, 4)}%`;

        return (
          <div className="barRow" key={label}>
            <div className="barRowTop">
              <span className="barLabel">{label}</span>
              <span className="barValue">{valueFormatter(value)}</span>
            </div>
            <div className="barTrack">
              <span className="barFill" style={{ width }} />
            </div>
          </div>
        );
      })}
    </div>
  );
}

function TrendChart({ rows }: { rows: MonthlyPoint[] }) {
  const width = 760;
  const height = 260;
  const paddingX = 36;
  const paddingY = 28;
  const chartWidth = width - paddingX * 2;
  const chartHeight = height - paddingY * 2;
  const maxValue = Math.max(...rows.map(row => Math.max(row.revenue, row.rollingAverage)), 1);

  const toPoint = (value: number, index: number) => {
    const x = rows.length === 1 ? width / 2 : paddingX + (chartWidth * index) / (rows.length - 1);
    const y = height - paddingY - (value / maxValue) * chartHeight;
    return `${x},${y}`;
  };

  const revenuePoints = rows.map((row, index) => toPoint(row.revenue, index)).join(' ');
  const averagePoints = rows.map((row, index) => toPoint(row.rollingAverage, index)).join(' ');

  return (
    <div className="trendChart">
      <svg viewBox={`0 0 ${width} ${height}`} role="img" aria-label="Monthly revenue trend chart">
        <defs>
          <linearGradient id="trendFill" x1="0" x2="0" y1="0" y2="1">
            <stop offset="0%" stopColor="rgba(6, 182, 212, 0.35)" />
            <stop offset="100%" stopColor="rgba(6, 182, 212, 0)" />
          </linearGradient>
        </defs>
        <line x1={paddingX} y1={height - paddingY} x2={width - paddingX} y2={height - paddingY} className="chartAxis" />
        <polyline points={revenuePoints} className="trendLine" />
        <polyline points={averagePoints} className="trendAverage" />
        {rows.map((row, index) => {
          const [x, y] = toPoint(row.revenue, index).split(',').map(Number);
          return <circle key={row.month} cx={x} cy={y} r="4" className="trendDot" />;
        })}
      </svg>

      <div className="trendLabels">
        {rows.map(row => (
          <div className="trendLabel" key={row.month}>
            <span>{formatMonthLabel(row.month)}</span>
            <strong>{formatCompactCurrency(row.revenue)}</strong>
          </div>
        ))}
      </div>
    </div>
  );
}

function TopProductsTable({ rows }: { rows: ProductPoint[] }) {
  return (
    <div className="tableWrap">
      <table className="dataTable">
        <thead>
          <tr>
            <th>Product</th>
            <th>Category</th>
            <th>Revenue</th>
            <th>Units</th>
            <th>Orders</th>
          </tr>
        </thead>
        <tbody>
          {rows.map(row => (
            <tr key={row.product}>
              <td>{row.product}</td>
              <td>{row.category}</td>
              <td>{formatCurrency(row.totalRevenue)}</td>
              <td>{row.totalUnits}</td>
              <td>{row.orders}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default function Index() {
  const [payload, setPayload] = useState<SalesPayload | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [filters, setFilters] = useState<Filters>({
    region: '',
    dateFrom: '',
    dateTo: '',
  });

  useEffect(() => {
    let alive = true;

    fetchSales()
      .then(data => {
        if (alive) {
          setPayload(data);
        }
      })
      .catch((err: Error) => {
        if (alive) {
          setError(err.message);
        }
      })
      .finally(() => {
        if (alive) {
          setLoading(false);
        }
      });

    return () => {
      alive = false;
    };
  }, []);

  const refresh = async (nextFilters?: Filters) => {
    setLoading(true);
    setError('');
    try {
      const data = await fetchSales(nextFilters);
      setPayload(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load sales data');
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    void refresh(filters);
  };

  const handleReset = () => {
    const emptyFilters = { region: '', dateFrom: '', dateTo: '' };
    setFilters(emptyFilters);
    void refresh(emptyFilters);
  };

  const regionOptions = payload?.filters.regions || DEFAULT_REGIONS;
  const summary = payload?.summary;
  const graphs = payload?.graphs;

  return (
    <main className="page">
      <section className="hero">
        <div className="heroCopy">
          <p className="eyebrow">NumPy + pandas dashboard</p>
          <h1>Sales Dashboard</h1>
          <p className="subtitle">
            The Python API builds chart-ready aggregates from generated sales data.
            The frontend fetches them and renders the graphs directly.
          </p>
        </div>

        <form className="filtersPanel" onSubmit={handleSubmit}>
          <div className="filtersGrid">
            <label className="field">
              <span>Region</span>
              <select
                value={filters.region}
                onChange={e => setFilters(current => ({ ...current, region: e.target.value }))}
              >
                <option value="">All regions</option>
                {regionOptions.map(region => (
                  <option key={region} value={region}>{region}</option>
                ))}
              </select>
            </label>

            <label className="field">
              <span>From</span>
              <input
                type="date"
                value={filters.dateFrom}
                onChange={e => setFilters(current => ({ ...current, dateFrom: e.target.value }))}
              />
            </label>

            <label className="field">
              <span>To</span>
              <input
                type="date"
                value={filters.dateTo}
                onChange={e => setFilters(current => ({ ...current, dateTo: e.target.value }))}
              />
            </label>
          </div>

          <div className="filtersActions">
            <button type="submit">Update charts</button>
            <button type="button" className="ghostButton" onClick={handleReset}>
              Clear filters
            </button>
          </div>
        </form>
      </section>

      {error && <p className="error">{error}</p>}

      <section className="metricGrid">
        <MetricCard
          label="Total revenue"
          value={summary ? formatCurrency(summary.totalRevenue) : 'Loading...'}
          hint="All filtered orders"
        />
        <MetricCard
          label="Average order"
          value={summary ? formatCurrency(summary.avgOrder) : 'Loading...'}
          hint="Mean revenue per order"
        />
        <MetricCard
          label="Orders"
          value={summary ? summary.totalOrders.toString() : 'Loading...'}
          hint="Generated rows in the dataset"
        />
        <MetricCard
          label="Units sold"
          value={summary ? summary.totalUnits.toString() : 'Loading...'}
          hint="Total item count"
        />
      </section>

      <section className="chartGrid">
        <article className="card chartCard wide">
          <div className="cardHeader">
            <div>
              <h2>Monthly revenue trend</h2>
              <p>Revenue line plus a 3-month rolling average.</p>
            </div>
          </div>
          {loading && !graphs ? (
            <p className="loading">Loading chart data...</p>
          ) : graphs?.monthlyRevenue?.length ? (
            <TrendChart rows={graphs.monthlyRevenue} />
          ) : (
            <p className="loading">No data for the selected filters.</p>
          )}
        </article>

        <article className="card chartCard">
          <div className="cardHeader">
            <div>
              <h2>Revenue by category</h2>
              <p>Grouped from the pandas DataFrame.</p>
            </div>
          </div>
          {graphs?.categoryRevenue?.length ? (
            <HorizontalBarChart
              rows={graphs.categoryRevenue}
              labelKey="category"
              valueKey="revenue"
              valueFormatter={formatCurrency}
            />
          ) : (
            <p className="loading">No data available.</p>
          )}
        </article>

        <article className="card chartCard">
          <div className="cardHeader">
            <div>
              <h2>Revenue by region</h2>
              <p>Use the filter form to narrow the chart.</p>
            </div>
          </div>
          {graphs?.regionRevenue?.length ? (
            <HorizontalBarChart
              rows={graphs.regionRevenue}
              labelKey="region"
              valueKey="revenue"
              valueFormatter={formatCurrency}
            />
          ) : (
            <p className="loading">No data available.</p>
          )}
        </article>
      </section>

      <section className="card chartCard tableCard">
        <div className="cardHeader">
          <div>
            <h2>Top products</h2>
            <p>Sorted by total revenue with pandas groupby aggregations.</p>
          </div>
        </div>
        {graphs?.topProducts?.length ? (
          <TopProductsTable rows={graphs.topProducts} />
        ) : (
          <p className="loading">No data available.</p>
        )}
      </section>
    </main>
  );
}
