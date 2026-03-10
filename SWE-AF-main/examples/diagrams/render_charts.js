/**
 * render_charts.js — D3 + jsdom headless SVG renderer
 *
 * Usage: deno run --allow-read --allow-write render_charts.js <data.json> <output_dir>
 *
 * Reads pre-computed chart data from JSON and writes 14 SVG files.
 */

import * as d3 from "npm:d3@7";
import { JSDOM } from "npm:jsdom@25";

// ── Style constants (visx / Airbnb aesthetic) ────────────────────────
const W = 1200;
const H = 700;
const FONT = '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif';
const TITLE_SIZE = 18;
const LABEL_SIZE = 12;
const AXIS_COLOR = "#888";
const GRID_COLOR = "#f0f0f0";
const BG = "#fff";
const MARGIN = { top: 70, right: 50, bottom: 70, left: 100 };

// ── Helpers ──────────────────────────────────────────────────────────

function createSVG(w = W, h = H) {
  const dom = new JSDOM("<!DOCTYPE html><html><body></body></html>");
  const document = dom.window.document;
  const body = d3.select(document.body);

  const svg = body
    .append("svg")
    .attr("xmlns", "http://www.w3.org/2000/svg")
    .attr("viewBox", `0 0 ${w} ${h}`)
    .attr("width", w)
    .attr("height", h)
    .style("font-family", FONT);

  // white background
  svg.append("rect").attr("width", w).attr("height", h).attr("fill", BG);

  return {
    svg,
    serialize: () => body.html(),
  };
}

function addTitle(svg, text, w = W) {
  svg
    .append("text")
    .attr("x", w / 2)
    .attr("y", 35)
    .attr("text-anchor", "middle")
    .attr("font-size", TITLE_SIZE)
    .attr("font-weight", 700)
    .attr("fill", "#222")
    .text(text);
}

function addSubtitle(svg, text, w = W) {
  svg
    .append("text")
    .attr("x", w / 2)
    .attr("y", 55)
    .attr("text-anchor", "middle")
    .attr("font-size", 13)
    .attr("fill", "#666")
    .text(text);
}

function addXAxis(svg, scale, y, label, w = W) {
  const g = svg
    .append("g")
    .attr("transform", `translate(0,${y})`);

  const axis = d3.axisBottom(scale).ticks(8);
  g.call(axis);
  g.selectAll("line").attr("stroke", AXIS_COLOR);
  g.selectAll("path").attr("stroke", AXIS_COLOR);
  g.selectAll("text").attr("fill", AXIS_COLOR).attr("font-size", 11);

  if (label) {
    svg
      .append("text")
      .attr("x", (MARGIN.left + w - MARGIN.right) / 2)
      .attr("y", y + 45)
      .attr("text-anchor", "middle")
      .attr("font-size", LABEL_SIZE)
      .attr("fill", "#555")
      .text(label);
  }
}

function addYAxis(svg, scale, x, label) {
  const g = svg
    .append("g")
    .attr("transform", `translate(${x},0)`);

  const axis = d3.axisLeft(scale).ticks(6);
  g.call(axis);
  g.selectAll("line").attr("stroke", AXIS_COLOR);
  g.selectAll("path").attr("stroke", AXIS_COLOR);
  g.selectAll("text").attr("fill", AXIS_COLOR).attr("font-size", 11);

  if (label) {
    svg
      .append("text")
      .attr("transform", `rotate(-90)`)
      .attr("x", -(MARGIN.top + H - MARGIN.bottom) / 2)
      .attr("y", x - 45)
      .attr("text-anchor", "middle")
      .attr("font-size", LABEL_SIZE)
      .attr("fill", "#555")
      .text(label);
  }
}

function addGridY(svg, scale, x0, x1) {
  const ticks = scale.ticks(6);
  ticks.forEach((t) => {
    svg
      .append("line")
      .attr("x1", x0)
      .attr("x2", x1)
      .attr("y1", scale(t))
      .attr("y2", scale(t))
      .attr("stroke", GRID_COLOR)
      .attr("stroke-width", 1);
  });
}

function addGridX(svg, scale, y0, y1) {
  const ticks = scale.ticks(8);
  ticks.forEach((t) => {
    svg
      .append("line")
      .attr("x1", scale(t))
      .attr("x2", scale(t))
      .attr("y1", y0)
      .attr("y2", y1)
      .attr("stroke", GRID_COLOR)
      .attr("stroke-width", 1);
  });
}

function truncate(s, maxLen = 20) {
  return s.length > maxLen ? s.slice(0, maxLen - 1) + "…" : s;
}

// ── Chart 01: Cost Allocation Treemap ────────────────────────────────
function chart01(data, palette) {
  const { svg, serialize } = createSVG();
  addTitle(svg, "Where does the money go?");

  const root = d3
    .hierarchy({ children: data.cost_treemap })
    .sum((d) => d.cost_usd);

  d3.treemap()
    .size([W - 40, H - 90])
    .padding(3)
    .round(true)(root);

  const g = svg.append("g").attr("transform", "translate(20,70)");

  g.selectAll("rect")
    .data(root.leaves())
    .join("rect")
    .attr("x", (d) => d.x0)
    .attr("y", (d) => d.y0)
    .attr("width", (d) => d.x1 - d.x0)
    .attr("height", (d) => d.y1 - d.y0)
    .attr("fill", (d) => palette[d.data.category] || "#888")
    .attr("rx", 4)
    .attr("opacity", 0.9);

  g.selectAll("text.name")
    .data(root.leaves())
    .join("text")
    .attr("class", "name")
    .attr("x", (d) => d.x0 + 6)
    .attr("y", (d) => d.y0 + 18)
    .attr("font-size", (d) => {
      const area = (d.x1 - d.x0) * (d.y1 - d.y0);
      return area > 15000 ? 13 : area > 5000 ? 11 : 9;
    })
    .attr("fill", "#fff")
    .attr("font-weight", 600)
    .text((d) => {
      const w = d.x1 - d.x0;
      return w > 60 ? d.data.category : "";
    });

  g.selectAll("text.val")
    .data(root.leaves())
    .join("text")
    .attr("class", "val")
    .attr("x", (d) => d.x0 + 6)
    .attr("y", (d) => d.y0 + 34)
    .attr("font-size", 11)
    .attr("fill", "rgba(255,255,255,0.85)")
    .text((d) => {
      const w = d.x1 - d.x0;
      return w > 60 ? `$${d.data.cost_usd.toFixed(2)}` : "";
    });

  return serialize();
}

// ── Chart 02: Time Allocation Treemap ────────────────────────────────
function chart02(data, palette) {
  const { svg, serialize } = createSVG();
  addTitle(svg, "Where does the time go?");

  const root = d3
    .hierarchy({ children: data.time_treemap })
    .sum((d) => d.duration_min);

  d3.treemap()
    .size([W - 40, H - 90])
    .padding(3)
    .round(true)(root);

  const g = svg.append("g").attr("transform", "translate(20,70)");

  g.selectAll("rect")
    .data(root.leaves())
    .join("rect")
    .attr("x", (d) => d.x0)
    .attr("y", (d) => d.y0)
    .attr("width", (d) => d.x1 - d.x0)
    .attr("height", (d) => d.y1 - d.y0)
    .attr("fill", (d) => palette[d.data.category] || "#888")
    .attr("rx", 4)
    .attr("opacity", 0.9);

  g.selectAll("text.name")
    .data(root.leaves())
    .join("text")
    .attr("class", "name")
    .attr("x", (d) => d.x0 + 6)
    .attr("y", (d) => d.y0 + 18)
    .attr("font-size", (d) => {
      const area = (d.x1 - d.x0) * (d.y1 - d.y0);
      return area > 15000 ? 13 : area > 5000 ? 11 : 9;
    })
    .attr("fill", "#fff")
    .attr("font-weight", 600)
    .text((d) => {
      const w = d.x1 - d.x0;
      return w > 60 ? d.data.category : "";
    });

  g.selectAll("text.val")
    .data(root.leaves())
    .join("text")
    .attr("class", "val")
    .attr("x", (d) => d.x0 + 6)
    .attr("y", (d) => d.y0 + 34)
    .attr("font-size", 11)
    .attr("fill", "rgba(255,255,255,0.85)")
    .text((d) => {
      const w = d.x1 - d.x0;
      return w > 60 ? `${d.data.duration_min.toFixed(1)}m` : "";
    });

  return serialize();
}

// ── Chart 03: Burn Rate (Area) ───────────────────────────────────────
function chart03(data) {
  const { svg, serialize } = createSVG();
  addTitle(svg, "How fast are we spending?");

  const pts = data.burn_rate;
  const xMax = d3.max(pts, (d) => d.elapsed_min);
  const yMax = d3.max(pts, (d) => d.cum_cost);

  const x = d3.scaleLinear().domain([0, xMax]).range([MARGIN.left, W - MARGIN.right]);
  const y = d3.scaleLinear().domain([0, yMax * 1.05]).range([H - MARGIN.bottom, MARGIN.top]);

  addGridY(svg, y, MARGIN.left, W - MARGIN.right);
  addGridX(svg, x, MARGIN.top, H - MARGIN.bottom);

  const area = d3
    .area()
    .x((d) => x(d.elapsed_min))
    .y0(H - MARGIN.bottom)
    .y1((d) => y(d.cum_cost));

  svg
    .append("path")
    .datum(pts)
    .attr("d", area)
    .attr("fill", "rgba(225,87,89,0.15)");

  const line = d3
    .line()
    .x((d) => x(d.elapsed_min))
    .y((d) => y(d.cum_cost));

  svg
    .append("path")
    .datum(pts)
    .attr("d", line)
    .attr("fill", "none")
    .attr("stroke", "#e15759")
    .attr("stroke-width", 2.5);

  addXAxis(svg, x, H - MARGIN.bottom, "Elapsed time (minutes)");
  addYAxis(svg, y, MARGIN.left, "Cumulative cost (USD)");

  return serialize();
}

// ── Chart 04: Parallelism (Step Area) ────────────────────────────────
function chart04(data) {
  const { svg, serialize } = createSVG();
  addTitle(svg, "How many agents run at once?");

  const pts = data.parallelism;
  const xMax = d3.max(pts, (d) => d.time_min);
  const yMax = d3.max(pts, (d) => d.concurrent);

  const x = d3.scaleLinear().domain([0, xMax]).range([MARGIN.left, W - MARGIN.right]);
  const y = d3.scaleLinear().domain([0, yMax + 1]).range([H - MARGIN.bottom, MARGIN.top]);

  addGridY(svg, y, MARGIN.left, W - MARGIN.right);

  const area = d3
    .area()
    .curve(d3.curveStepAfter)
    .x((d) => x(d.time_min))
    .y0(H - MARGIN.bottom)
    .y1((d) => y(d.concurrent));

  svg
    .append("path")
    .datum(pts)
    .attr("d", area)
    .attr("fill", "rgba(78,121,167,0.2)");

  const line = d3
    .line()
    .curve(d3.curveStepAfter)
    .x((d) => x(d.time_min))
    .y((d) => y(d.concurrent));

  svg
    .append("path")
    .datum(pts)
    .attr("d", line)
    .attr("fill", "none")
    .attr("stroke", "#4e79a7")
    .attr("stroke-width", 2);

  addXAxis(svg, x, H - MARGIN.bottom, "Elapsed time (minutes)");
  addYAxis(svg, y, MARGIN.left, "Concurrent agents");

  return serialize();
}

// ── Chart 05: Cost Efficiency Bubble ─────────────────────────────────
function chart05(data, palette) {
  const { svg, serialize } = createSVG();
  addTitle(svg, "Which phases cost the most per minute?");
  addSubtitle(svg, "bubble size = $/min");

  const pts = data.cost_efficiency;
  const xMax = d3.max(pts, (d) => d.total_min);
  const yMax = d3.max(pts, (d) => d.total_cost);
  const rMax = d3.max(pts, (d) => d.cost_per_min);

  const x = d3.scaleLinear().domain([0, xMax * 1.1]).range([MARGIN.left, W - MARGIN.right]);
  const y = d3.scaleLinear().domain([0, yMax * 1.1]).range([H - MARGIN.bottom, MARGIN.top]);
  const r = d3.scaleSqrt().domain([0, rMax]).range([6, 40]);

  addGridY(svg, y, MARGIN.left, W - MARGIN.right);
  addGridX(svg, x, MARGIN.top, H - MARGIN.bottom);

  pts.forEach((d) => {
    svg
      .append("circle")
      .attr("cx", x(d.total_min))
      .attr("cy", y(d.total_cost))
      .attr("r", r(d.cost_per_min))
      .attr("fill", palette[d.category] || "#888")
      .attr("opacity", 0.75)
      .attr("stroke", "#fff")
      .attr("stroke-width", 1.5);

    svg
      .append("text")
      .attr("x", x(d.total_min))
      .attr("y", y(d.total_cost) - r(d.cost_per_min) - 6)
      .attr("text-anchor", "middle")
      .attr("font-size", 10)
      .attr("fill", "#444")
      .text(d.category);
  });

  addXAxis(svg, x, H - MARGIN.bottom, "Total time (minutes)");
  addYAxis(svg, y, MARGIN.left, "Total cost (USD)");

  return serialize();
}

// ── Chart 06: Time Heatmap ───────────────────────────────────────────
function chart06(data) {
  const hm = data.time_heatmap;
  const nRows = hm.issues.length;
  const nCols = hm.categories.length;
  const cellH = Math.max(28, Math.min(50, (H - 120) / nRows));
  const cellW = Math.max(80, Math.min(160, (W - 200) / nCols));
  const totalH = Math.max(H, nRows * cellH + 150);
  const { svg, serialize } = createSVG(W, totalH);
  addTitle(svg, "Where do issues get stuck?", W);
  addSubtitle(svg, "minutes per phase", W);

  const flat = [];
  let maxVal = 0;
  hm.values.forEach((row, i) =>
    row.forEach((v, j) => {
      flat.push({ row: i, col: j, val: v });
      if (v > maxVal) maxVal = v;
    })
  );

  const color = d3.scaleSequential(d3.interpolateYlOrRd).domain([0, maxVal]);

  const xOff = 160;
  const yOff = 80;

  // column labels
  hm.categories.forEach((cat, j) => {
    svg
      .append("text")
      .attr("x", xOff + j * cellW + cellW / 2)
      .attr("y", yOff - 8)
      .attr("text-anchor", "middle")
      .attr("font-size", 11)
      .attr("fill", "#555")
      .text(cat);
  });

  // row labels
  hm.issues.forEach((issue, i) => {
    svg
      .append("text")
      .attr("x", xOff - 8)
      .attr("y", yOff + i * cellH + cellH / 2 + 4)
      .attr("text-anchor", "end")
      .attr("font-size", 10)
      .attr("fill", "#555")
      .text(truncate(issue, 22));
  });

  // cells
  flat.forEach((d) => {
    svg
      .append("rect")
      .attr("x", xOff + d.col * cellW)
      .attr("y", yOff + d.row * cellH)
      .attr("width", cellW - 2)
      .attr("height", cellH - 2)
      .attr("fill", d.val > 0 ? color(d.val) : "#fafafa")
      .attr("rx", 3);

    if (d.val > 0) {
      svg
        .append("text")
        .attr("x", xOff + d.col * cellW + cellW / 2 - 1)
        .attr("y", yOff + d.row * cellH + cellH / 2 + 4)
        .attr("text-anchor", "middle")
        .attr("font-size", 10)
        .attr("fill", d.val > maxVal * 0.6 ? "#fff" : "#333")
        .text(d.val.toFixed(1));
    }
  });

  return serialize();
}

// ── Chart 07: Cost Heatmap ───────────────────────────────────────────
function chart07(data) {
  const hm = data.cost_heatmap;
  const nRows = hm.issues.length;
  const nCols = hm.categories.length;
  const cellH = Math.max(28, Math.min(50, (H - 120) / nRows));
  const cellW = Math.max(80, Math.min(160, (W - 200) / nCols));
  const totalH = Math.max(H, nRows * cellH + 150);
  const { svg, serialize } = createSVG(W, totalH);
  addTitle(svg, "Where does each issue spend money?", W);

  const flat = [];
  let maxVal = 0;
  hm.values.forEach((row, i) =>
    row.forEach((v, j) => {
      flat.push({ row: i, col: j, val: v });
      if (v > maxVal) maxVal = v;
    })
  );

  const color = d3.scaleSequential(d3.interpolateYlOrRd).domain([0, maxVal]);

  const xOff = 160;
  const yOff = 80;

  hm.categories.forEach((cat, j) => {
    svg
      .append("text")
      .attr("x", xOff + j * cellW + cellW / 2)
      .attr("y", yOff - 8)
      .attr("text-anchor", "middle")
      .attr("font-size", 11)
      .attr("fill", "#555")
      .text(cat);
  });

  hm.issues.forEach((issue, i) => {
    svg
      .append("text")
      .attr("x", xOff - 8)
      .attr("y", yOff + i * cellH + cellH / 2 + 4)
      .attr("text-anchor", "end")
      .attr("font-size", 10)
      .attr("fill", "#555")
      .text(truncate(issue, 22));
  });

  flat.forEach((d) => {
    svg
      .append("rect")
      .attr("x", xOff + d.col * cellW)
      .attr("y", yOff + d.row * cellH)
      .attr("width", cellW - 2)
      .attr("height", cellH - 2)
      .attr("fill", d.val > 0 ? color(d.val) : "#fafafa")
      .attr("rx", 3);

    if (d.val > 0) {
      svg
        .append("text")
        .attr("x", xOff + d.col * cellW + cellW / 2 - 1)
        .attr("y", yOff + d.row * cellH + cellH / 2 + 4)
        .attr("text-anchor", "middle")
        .attr("font-size", 10)
        .attr("fill", d.val > maxVal * 0.6 ? "#fff" : "#333")
        .text(`$${d.val.toFixed(2)}`);
    }
  });

  return serialize();
}

// ── Chart 08: Duration Violin ────────────────────────────────────────
function chart08(data, palette, phaseOrder) {
  const { svg, serialize } = createSVG(1400, H);
  addTitle(svg, "How variable is each phase?", 1400);

  const pts = data.duration_violin;

  // group by category
  const groups = {};
  pts.forEach((d) => {
    if (!groups[d.category]) groups[d.category] = [];
    groups[d.category].push(d.duration_min);
  });

  // only cats with >=2 data points
  const cats = phaseOrder.filter((c) => groups[c] && groups[c].length >= 2);

  const bandW = (1400 - 140) / (cats.length || 1);
  const margin = { top: 70, bottom: 70, left: 80, right: 60 };

  const yMax = d3.max(pts, (d) => d.duration_min) * 1.1;
  const y = d3.scaleLinear().domain([0, yMax]).range([H - margin.bottom, margin.top]);

  addGridY(svg, y, margin.left, 1400 - margin.right);

  cats.forEach((cat, i) => {
    const values = groups[cat].sort(d3.ascending);
    const cx = margin.left + i * bandW + bandW / 2;
    const halfW = bandW * 0.35;

    // KDE: use d3.bin
    const nBins = Math.min(20, Math.max(5, Math.ceil(values.length / 2)));
    const bins = d3
      .bin()
      .domain([0, yMax])
      .thresholds(nBins)(values);

    const maxCount = d3.max(bins, (b) => b.length);
    const wScale = d3.scaleLinear().domain([0, maxCount]).range([0, halfW]);

    // mirror area
    const areaRight = d3
      .area()
      .curve(d3.curveBasis)
      .y((b) => y((b.x0 + b.x1) / 2))
      .x0(cx)
      .x1((b) => cx + wScale(b.length));

    const areaLeft = d3
      .area()
      .curve(d3.curveBasis)
      .y((b) => y((b.x0 + b.x1) / 2))
      .x0(cx)
      .x1((b) => cx - wScale(b.length));

    svg
      .append("path")
      .datum(bins)
      .attr("d", areaRight)
      .attr("fill", palette[cat] || "#888")
      .attr("opacity", 0.6);

    svg
      .append("path")
      .datum(bins)
      .attr("d", areaLeft)
      .attr("fill", palette[cat] || "#888")
      .attr("opacity", 0.6);

    // individual points (jittered)
    values.forEach((v) => {
      const jitter = (Math.random() - 0.5) * halfW * 0.4;
      svg
        .append("circle")
        .attr("cx", cx + jitter)
        .attr("cy", y(v))
        .attr("r", 3)
        .attr("fill", palette[cat] || "#888")
        .attr("opacity", 0.5);
    });

    // median line
    const med = d3.median(values);
    svg
      .append("line")
      .attr("x1", cx - halfW * 0.5)
      .attr("x2", cx + halfW * 0.5)
      .attr("y1", y(med))
      .attr("y2", y(med))
      .attr("stroke", "#222")
      .attr("stroke-width", 2);

    // label
    svg
      .append("text")
      .attr("x", cx)
      .attr("y", H - margin.bottom + 20)
      .attr("text-anchor", "middle")
      .attr("font-size", 10)
      .attr("fill", "#555")
      .attr("transform", `rotate(-35, ${cx}, ${H - margin.bottom + 20})`)
      .text(cat);
  });

  // Y axis
  addYAxis(svg, y, margin.left, "Duration (minutes)");

  return serialize();
}

// ── Chart 09: Effort Scatter ─────────────────────────────────────────
function chart09(data, palette) {
  const { svg, serialize } = createSVG();
  addTitle(svg, "Thinking vs doing?");
  addSubtitle(svg, "size = cost");

  const pts = data.effort_scatter;
  const xMax = d3.max(pts, (d) => d.num_turns) * 1.1;
  const yMax = d3.max(pts, (d) => d.tool_calls) * 1.1;
  const sMax = d3.max(pts, (d) => d.cost_usd);

  const x = d3.scaleLinear().domain([0, xMax]).range([MARGIN.left, W - MARGIN.right]);
  const y = d3.scaleLinear().domain([0, yMax]).range([H - MARGIN.bottom, MARGIN.top]);
  const s = d3.scaleSqrt().domain([0, sMax]).range([3, 25]);

  addGridY(svg, y, MARGIN.left, W - MARGIN.right);
  addGridX(svg, x, MARGIN.top, H - MARGIN.bottom);

  pts.forEach((d) => {
    svg
      .append("circle")
      .attr("cx", x(d.num_turns))
      .attr("cy", y(d.tool_calls))
      .attr("r", s(d.cost_usd))
      .attr("fill", palette[d.category] || "#888")
      .attr("opacity", 0.65)
      .attr("stroke", "#fff")
      .attr("stroke-width", 0.8);
  });

  addXAxis(svg, x, H - MARGIN.bottom, "Turns (thinking)");
  addYAxis(svg, y, MARGIN.left, "Tool calls (doing)");

  // legend
  const cats = [...new Set(pts.map((d) => d.category))];
  const legendG = svg.append("g").attr("transform", `translate(${W - 170}, ${MARGIN.top + 10})`);
  cats.slice(0, 12).forEach((cat, i) => {
    legendG
      .append("circle")
      .attr("cx", 0)
      .attr("cy", i * 18)
      .attr("r", 5)
      .attr("fill", palette[cat] || "#888");
    legendG
      .append("text")
      .attr("x", 12)
      .attr("y", i * 18 + 4)
      .attr("font-size", 10)
      .attr("fill", "#555")
      .text(cat);
  });

  return serialize();
}

// ── Chart 10: Pipeline Flow (Gantt) ──────────────────────────────────
function chart10(data, palette, phaseOrder) {
  const pts = data.pipeline_flow;
  const usedCats = phaseOrder.filter((c) => pts.some((p) => p.category === c));
  const totalH = Math.max(H, usedCats.length * 55 + 150);
  const { svg, serialize } = createSVG(W, totalH);
  addTitle(svg, "When does each stage run?", W);

  const xMax = d3.max(pts, (d) => d.start_min + d.dur_min);
  const x = d3.scaleLinear().domain([0, xMax]).range([MARGIN.left, W - MARGIN.right]);

  const bandH = (totalH - 150) / usedCats.length;
  const yMap = {};
  usedCats.forEach((c, i) => {
    yMap[c] = 90 + i * bandH;
  });

  addGridX(svg, x, 80, totalH - 60);

  // lane labels
  usedCats.forEach((cat) => {
    svg
      .append("text")
      .attr("x", MARGIN.left - 8)
      .attr("y", yMap[cat] + bandH / 2)
      .attr("text-anchor", "end")
      .attr("font-size", 11)
      .attr("fill", "#555")
      .text(cat);
  });

  // bars
  pts.forEach((d) => {
    if (yMap[d.category] === undefined) return;
    svg
      .append("rect")
      .attr("x", x(d.start_min))
      .attr("y", yMap[d.category] + bandH * 0.15)
      .attr("width", Math.max(2, x(d.start_min + d.dur_min) - x(d.start_min)))
      .attr("height", bandH * 0.7)
      .attr("fill", palette[d.category] || "#888")
      .attr("opacity", 0.85)
      .attr("rx", 3);
  });

  addXAxis(svg, x, totalH - 60, "Elapsed time (minutes)", W);

  return serialize();
}

// ── Chart 11: Parallelism Ratio (Horiz Bar) ──────────────────────────
function chart11(data, palette) {
  const pts = data.parallelism_ratio;
  const { svg, serialize } = createSVG();
  addTitle(svg, "Are we using concurrency well?");
  addSubtitle(svg, ">1 = parallel");

  const xMax = d3.max(pts, (d) => d.ratio) * 1.15;
  const x = d3.scaleLinear().domain([0, xMax]).range([MARGIN.left, W - MARGIN.right]);
  const bandH = (H - MARGIN.top - MARGIN.bottom) / pts.length;

  addGridX(svg, x, MARGIN.top, H - MARGIN.bottom);

  pts.forEach((d, i) => {
    const yPos = MARGIN.top + i * bandH;
    svg
      .append("rect")
      .attr("x", x(0))
      .attr("y", yPos + bandH * 0.15)
      .attr("width", x(d.ratio) - x(0))
      .attr("height", bandH * 0.7)
      .attr("fill", palette[d.category] || "#888")
      .attr("opacity", 0.85)
      .attr("rx", 3);

    svg
      .append("text")
      .attr("x", MARGIN.left - 8)
      .attr("y", yPos + bandH / 2 + 4)
      .attr("text-anchor", "end")
      .attr("font-size", 11)
      .attr("fill", "#555")
      .text(d.category);

    svg
      .append("text")
      .attr("x", x(d.ratio) + 6)
      .attr("y", yPos + bandH / 2 + 4)
      .attr("font-size", 11)
      .attr("fill", "#444")
      .text(d.ratio.toFixed(2));
  });

  // reference line at x=1
  svg
    .append("line")
    .attr("x1", x(1))
    .attr("x2", x(1))
    .attr("y1", MARGIN.top)
    .attr("y2", H - MARGIN.bottom)
    .attr("stroke", "#888")
    .attr("stroke-width", 1)
    .attr("stroke-dasharray", "5,4");

  svg
    .append("text")
    .attr("x", x(1) + 4)
    .attr("y", MARGIN.top - 5)
    .attr("font-size", 10)
    .attr("fill", "#888")
    .text("1.0");

  return serialize();
}

// ── Chart 12: Issue Ranking (Stacked Bar) ────────────────────────────
function chart12(data, palette) {
  const pts = data.issue_ranking;
  const phases = pts.length > 0 ? Object.keys(pts[0].phase_costs) : [];
  const totalH = Math.max(H, pts.length * 35 + 150);
  const { svg, serialize } = createSVG(W, totalH);
  addTitle(svg, "Which issues cost the most?", W);

  // compute totals for xMax
  let xMax = 0;
  pts.forEach((d) => {
    const total = Object.values(d.phase_costs).reduce((a, b) => a + b, 0);
    if (total > xMax) xMax = total;
  });
  xMax *= 1.1;

  const x = d3.scaleLinear().domain([0, xMax]).range([MARGIN.left, W - MARGIN.right]);
  const bandH = (totalH - 150) / pts.length;

  addGridX(svg, x, 80, totalH - 60);

  pts.forEach((d, i) => {
    const yPos = 90 + i * bandH;

    svg
      .append("text")
      .attr("x", MARGIN.left - 8)
      .attr("y", yPos + bandH / 2 + 4)
      .attr("text-anchor", "end")
      .attr("font-size", 10)
      .attr("fill", "#555")
      .text(truncate(d.issue, 18));

    let cumX = 0;
    phases.forEach((phase) => {
      const val = d.phase_costs[phase];
      if (val > 0) {
        svg
          .append("rect")
          .attr("x", x(cumX))
          .attr("y", yPos + bandH * 0.15)
          .attr("width", Math.max(0, x(cumX + val) - x(cumX)))
          .attr("height", bandH * 0.7)
          .attr("fill", palette[phase] || "#888")
          .attr("opacity", 0.85)
          .attr("rx", 2);
      }
      cumX += val;
    });
  });

  // legend
  const legendG = svg.append("g").attr("transform", `translate(${W - 220}, 60)`);
  phases.forEach((phase, i) => {
    legendG
      .append("rect")
      .attr("x", i * 100)
      .attr("y", 0)
      .attr("width", 12)
      .attr("height", 12)
      .attr("fill", palette[phase] || "#888")
      .attr("rx", 2);
    legendG
      .append("text")
      .attr("x", i * 100 + 16)
      .attr("y", 10)
      .attr("font-size", 10)
      .attr("fill", "#555")
      .text(phase);
  });

  addXAxis(svg, x, totalH - 60, "Cost (USD)", W);

  return serialize();
}

// ── Chart 13: Rework Cost (Vertical Bar) ─────────────────────────────
function chart13(data, palette) {
  const pts = data.rework;
  const { svg, serialize } = createSVG();
  addTitle(svg, "What's the cost of QA failure?");
  addSubtitle(svg, "red = review / QA rework");

  const yMax = d3.max(pts, (d) => d.cost_usd) * 1.1;
  const x = d3
    .scaleBand()
    .domain(pts.map((d) => d.category))
    .range([MARGIN.left, W - MARGIN.right])
    .padding(0.25);
  const y = d3.scaleLinear().domain([0, yMax]).range([H - MARGIN.bottom, MARGIN.top]);

  addGridY(svg, y, MARGIN.left, W - MARGIN.right);

  pts.forEach((d) => {
    svg
      .append("rect")
      .attr("x", x(d.category))
      .attr("y", y(d.cost_usd))
      .attr("width", x.bandwidth())
      .attr("height", H - MARGIN.bottom - y(d.cost_usd))
      .attr("fill", d.is_rework ? "#e15759" : palette[d.category] || "#888")
      .attr("opacity", 0.85)
      .attr("rx", 3);
  });

  // X axis labels (rotated)
  pts.forEach((d) => {
    svg
      .append("text")
      .attr("x", x(d.category) + x.bandwidth() / 2)
      .attr("y", H - MARGIN.bottom + 18)
      .attr("text-anchor", "end")
      .attr("font-size", 10)
      .attr("fill", "#555")
      .attr(
        "transform",
        `rotate(-35, ${x(d.category) + x.bandwidth() / 2}, ${H - MARGIN.bottom + 18})`
      )
      .text(d.category);
  });

  addYAxis(svg, y, MARGIN.left, "Cost (USD)");

  // legend
  const lg = svg.append("g").attr("transform", `translate(${W - 220}, ${MARGIN.top + 5})`);
  [
    { label: "Build", color: "#4e79a7" },
    { label: "Review / QA", color: "#e15759" },
  ].forEach((item, i) => {
    lg.append("rect")
      .attr("x", 0)
      .attr("y", i * 20)
      .attr("width", 14)
      .attr("height", 14)
      .attr("fill", item.color)
      .attr("rx", 2);
    lg.append("text")
      .attr("x", 20)
      .attr("y", i * 20 + 11)
      .attr("font-size", 11)
      .attr("fill", "#555")
      .text(item.label);
  });

  return serialize();
}

// ── Chart 14: Dashboard KPI Cards ────────────────────────────────────
function chart14(data) {
  const kpis = data.dashboard;
  const nCols = 3;
  const nRows = Math.ceil(kpis.length / nCols);
  const cardW = 340;
  const cardH = 140;
  const gap = 25;
  const totalW = nCols * cardW + (nCols - 1) * gap + 60;
  const totalH = nRows * cardH + (nRows - 1) * gap + 120;

  const { svg, serialize } = createSVG(totalW, totalH);
  addTitle(svg, "Key Metrics at a Glance", totalW);

  kpis.forEach((kpi, i) => {
    const col = i % nCols;
    const row = Math.floor(i / nCols);
    const cx = 30 + col * (cardW + gap);
    const cy = 80 + row * (cardH + gap);

    // card background
    svg
      .append("rect")
      .attr("x", cx)
      .attr("y", cy)
      .attr("width", cardW)
      .attr("height", cardH)
      .attr("fill", "#fafafa")
      .attr("rx", 10)
      .attr("stroke", "#eee")
      .attr("stroke-width", 1);

    // colored accent bar
    svg
      .append("rect")
      .attr("x", cx)
      .attr("y", cy)
      .attr("width", 5)
      .attr("height", cardH)
      .attr("fill", kpi.color)
      .attr("rx", 3);

    // value
    svg
      .append("text")
      .attr("x", cx + cardW / 2)
      .attr("y", cy + 65)
      .attr("text-anchor", "middle")
      .attr("font-size", 36)
      .attr("font-weight", 700)
      .attr("fill", kpi.color)
      .text(kpi.value);

    // label
    svg
      .append("text")
      .attr("x", cx + cardW / 2)
      .attr("y", cy + 100)
      .attr("text-anchor", "middle")
      .attr("font-size", 14)
      .attr("fill", "#666")
      .text(kpi.label);
  });

  return serialize();
}

// ── Main ─────────────────────────────────────────────────────────────
async function main() {
  const jsonPath = Deno.args[0];
  const outDir = Deno.args[1];

  if (!jsonPath || !outDir) {
    console.error("Usage: render_charts.js <data.json> <output_dir>");
    Deno.exit(1);
  }

  const raw = await Deno.readTextFile(jsonPath);
  const data = JSON.parse(raw);
  const palette = data.meta.palette;
  const phaseOrder = data.meta.phase_order;

  const charts = [
    ["01_cost_allocation.svg", () => chart01(data, palette)],
    ["02_time_allocation.svg", () => chart02(data, palette)],
    ["03_burn_rate.svg", () => chart03(data)],
    ["04_parallelism.svg", () => chart04(data)],
    ["05_cost_efficiency.svg", () => chart05(data, palette)],
    ["06_time_heatmap.svg", () => chart06(data)],
    ["07_cost_heatmap.svg", () => chart07(data)],
    ["08_duration_violin.svg", () => chart08(data, palette, phaseOrder)],
    ["09_effort_scatter.svg", () => chart09(data, palette)],
    ["10_pipeline_flow.svg", () => chart10(data, palette, phaseOrder)],
    ["11_parallelism_ratio.svg", () => chart11(data, palette)],
    ["12_issue_ranking.svg", () => chart12(data, palette)],
    ["13_rework.svg", () => chart13(data, palette)],
    ["14_dashboard.svg", () => chart14(data)],
  ];

  for (const [name, renderFn] of charts) {
    const svgStr = renderFn();
    const path = `${outDir}/${name}`;
    await Deno.writeTextFile(path, svgStr);
    console.log(`  ✓ ${name}`);
  }
}

main();
