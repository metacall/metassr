var zlib = require('zlib');

var MONTH_NAMES = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
var REGIONS = ['North', 'South', 'East', 'West'];
var PRODUCTS = [
  ['Widget', 'Electronics', 480],
  ['Gadget', 'Electronics', 320],
  ['Doohickey', 'Hardware', 210],
  ['Contraption', 'Hardware', 150],
  ['Thingamajig', 'Accessories', 90],
  ['Blender', 'Electronics', 260],
  ['Gear', 'Hardware', 120],
  ['Cable', 'Accessories', 40],
  ['Monitor', 'Electronics', 540],
  ['Screw', 'Hardware', 15],
  ['Sticker', 'Accessories', 8],
  ['Powerbank', 'Electronics', 380]
];
var N_RECORDS = 400;
var LCG_A = 1664525;
var LCG_C = 1013904223;
var LCG_M = 4294967296;

function dataset() {
  var state = 42;
  var months = new Array(12).fill(0);
  var totalRevenue = 0;
  var totalUnits = 0;
  for (var i = 0; i < N_RECORDS; i++) {
    state = (LCG_A * state + LCG_C) % LCG_M;
    var p = state;
    state = (LCG_A * state + LCG_C) % LCG_M;
    var q = state;
    var productIdx = p % PRODUCTS.length;
    var regionIdx = (p >>> 8) % REGIONS.length;
    var units = ((p >>> 16) % 50) + 1;
    var day = q % 365;
    var price = PRODUCTS[productIdx][2];
    var revenue = units * price;
    var month = Math.min(Math.floor(day / 30), 11);
    months[month] += revenue;
    totalRevenue += revenue;
    totalUnits += units;
  }
  return { months: months, totalRevenue: totalRevenue, totalUnits: totalUnits };
}

function crcTable() {
  var table = new Int32Array(256);
  for (var n = 0; n < 256; n++) {
    var c = n;
    for (var k = 0; k < 8; k++) {
      c = c & 1 ? 0xEDB88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c;
  }
  return table;
}

var CRC_TABLE = crcTable();

function crc32(buf) {
  var c = -1;
  for (var i = 0; i < buf.length; i++) {
    c = CRC_TABLE[(c ^ buf[i]) & 0xFF] ^ (c >>> 8);
  }
  return (c ^ -1) >>> 0;
}

function chunk(type, data) {
  var len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  var typeBuf = Buffer.from(type, 'ascii');
  var crcBuf = Buffer.alloc(4);
  crcBuf.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])), 0);
  return Buffer.concat([len, typeBuf, data, crcBuf]);
}

function encodePNG(width, height, rgba) {
  var sig = Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
  var ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;
  ihdr[10] = 0;
  ihdr[11] = 0;
  ihdr[12] = 0;
  var stride = width * 4;
  var raw = Buffer.alloc((stride + 1) * height);
  for (var y = 0; y < height; y++) {
    raw[y * (stride + 1)] = 0;
    rgba.copy(raw, y * (stride + 1) + 1, y * stride, (y + 1) * stride);
  }
  var idat = zlib.deflateSync(raw, { level: 9 });
  return Buffer.concat([sig, chunk('IHDR', ihdr), chunk('IDAT', idat), chunk('IEND', Buffer.alloc(0))]);
}

function Image(width, height) {
  this.width = width;
  this.height = height;
  this.data = Buffer.alloc(width * height * 4);
  this.data.fill(255);

  this.fillRect = function (x, y, w, h, color) {
    var x0 = Math.max(0, Math.floor(x));
    var y0 = Math.max(0, Math.floor(y));
    var x1 = Math.min(this.width, Math.ceil(x + w));
    var y1 = Math.min(this.height, Math.ceil(y + h));
    for (var yy = y0; yy < y1; yy++) {
      for (var xx = x0; xx < x1; xx++) {
        var o = (yy * this.width + xx) * 4;
        this.data[o] = color[0];
        this.data[o + 1] = color[1];
        this.data[o + 2] = color[2];
        this.data[o + 3] = color[3];
      }
    }
  };

  this.toPNG = function () {
    return encodePNG(this.width, this.height, this.data);
  };
}

var FONT = {
  A: [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
  B: [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
  C: [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
  D: [0x1C, 0x12, 0x11, 0x11, 0x11, 0x12, 0x1C],
  E: [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
  F: [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
  G: [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0F],
  H: [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
  I: [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
  J: [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0C],
  K: [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
  L: [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
  M: [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
  N: [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
  O: [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
  P: [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
  Q: [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
  R: [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
  S: [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
  T: [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
  U: [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
  V: [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
  W: [0x11, 0x11, 0x11, 0x15, 0x15, 0x1B, 0x11],
  X: [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
  Y: [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
  Z: [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
  '0': [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
  '1': [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
  '2': [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
  '3': [0x1E, 0x01, 0x01, 0x0E, 0x01, 0x01, 0x1E],
  '4': [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
  '5': [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x01, 0x1E],
  '6': [0x0E, 0x10, 0x10, 0x1E, 0x11, 0x11, 0x0E],
  '7': [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
  '8': [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
  '9': [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x01, 0x0E],
  '$': [0x04, 0x0F, 0x14, 0x0E, 0x05, 0x1E, 0x04],
  '.': [0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x06],
  ',': [0x00, 0x00, 0x00, 0x00, 0x06, 0x04, 0x08],
  '%': [0x19, 0x1A, 0x02, 0x04, 0x08, 0x0B, 0x13],
  '-': [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
  '+': [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
  ':': [0x00, 0x06, 0x06, 0x00, 0x06, 0x06, 0x00],
  '/': [0x01, 0x02, 0x02, 0x04, 0x08, 0x08, 0x10],
  ' ': [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
};

function textWidth(text, scale) {
  return (text.length * 6 - 1) * scale;
}

function fillText(img, text, x, y, scale, color) {
  var px = Math.round(x);
  for (var i = 0; i < text.length; i++) {
    var glyph = FONT[text[i]];
    if (glyph) {
      for (var r = 0; r < 7; r++) {
        var row = glyph[r];
        for (var c = 0; c < 5; c++) {
          if (row & (0x10 >> c)) {
            img.fillRect(px + c * scale, y + r * scale, scale, scale, color);
          }
        }
      }
    }
    px += 6 * scale;
  }
}

function format(n) {
  return n.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

function renderOG(page) {
  var data = dataset();
  var img = new Image(1200, 630);
  var canvas = [244, 241, 234, 255];
  var ink = [31, 41, 51, 255];
  var muted = [82, 97, 107, 255];
  var accent = [0, 109, 119, 255];
  var accentStrong = [0, 86, 94, 255];

  img.fillRect(0, 0, 1200, 630, canvas);
  img.fillRect(0, 0, 28, 630, accent);

  fillText(img, 'DATA DASHBOARD', 96, 74, 3, accent);
  fillText(img, 'SALES & REVENUE', 96, 114, 7, ink);
  fillText(img, 'NUMPY ANALYTICS + NODE OG IMAGE', 96, 212, 3, muted);

  var maxRev = Math.max.apply(null, data.months);
  var chartX = 96;
  var chartTop = 264;
  var chartBottom = 452;
  var chartW = 1008;
  var gap = 12;
  var barW = (chartW - 11 * gap) / 12;
  for (var m = 0; m < 12; m++) {
    var h = (data.months[m] / maxRev) * (chartBottom - chartTop);
    img.fillRect(chartX + m * (barW + gap), chartBottom - h, barW, h, m % 2 ? accent : accentStrong);
  }
  img.fillRect(chartX, chartBottom, chartW, 4, ink);
  for (m = 0; m < 12; m++) {
    fillText(img, MONTH_NAMES[m], chartX + m * (barW + gap) + Math.floor(barW / 2 - textWidth(MONTH_NAMES[m], 2) / 2), chartBottom + 16, 2, muted);
  }

  var kpiY = 522;
  fillText(img, 'TOTAL REVENUE', 96, kpiY, 3, muted);
  fillText(img, '$' + format(data.totalRevenue), 96, kpiY + 36, 4, ink);
  fillText(img, 'AVG ORDER', 456, kpiY, 3, muted);
  fillText(img, '$' + format(Math.round(data.totalRevenue / N_RECORDS)), 456, kpiY + 36, 4, ink);
  fillText(img, 'UNITS SOLD', 808, kpiY, 3, muted);
  fillText(img, format(data.totalUnits), 808, kpiY + 36, 4, ink);

  fillText(img, 'METASSR - ' + page.toUpperCase(), 96, 590, 2, accent);
  return img.toPNG();
}

function GET(req) {
  var reqObj = typeof req === 'string' ? JSON.parse(req) : req;
  var query = (reqObj && reqObj.query) || {};
  var page = query.page || 'dashboard';
  var png = renderOG(page);
  return JSON.stringify({
    status: 200,
    body: {
      page: page,
      width: 1200,
      height: 630,
      contentType: 'image/png',
      image: png.toString('base64'),
      generatedAt: new Date().toISOString()
    }
  });
}

module.exports = { GET: GET };