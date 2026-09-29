#!/usr/bin/env node
"use strict";

const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const PAYLOAD_PACKAGE = "@metassr/linux-x64-gnu";

function fail(message) {
	process.stderr.write(`metassr: ${message}\n`);
	process.exit(1);
}

function payloadDir() {
	try {
		return path.dirname(require.resolve(`${PAYLOAD_PACKAGE}/package.json`));
	} catch {
		if (process.platform !== "linux" || process.arch !== "x64") {
			fail(
				`no MetaSSR runtime available for ${process.platform}-${process.arch}: ` +
					`only linux-x64 (glibc) is published`
			);
		}
		fail(
			`the ${PAYLOAD_PACKAGE} package is missing. ` +
				`Reinstall without --no-optional: npm install -g metassr`
		);
	}
}

function libnodePath(payload) {
	const dir = path.join(payload, "node");
	const entry = fs
		.readdirSync(dir)
		.filter((name) => /^libnode\.so/.test(name))
		.sort()
		.pop();
	if (!entry) {
		fail(`runtime payload is incomplete: no libnode.so.* in ${dir}`);
	}
	return path.join(dir, entry);
}

function libpythonPath(payload) {
	const dir = path.join(payload, "lib");
	const entry = fs
		.readdirSync(dir)
		.filter((name) => /^libpython3\.\d+\.so/.test(name))
		.sort()
		.pop();
	if (!entry) {
		fail(`runtime payload is incomplete: no libpython3.*.so in ${dir}`);
	}
	return path.join(dir, entry);
}

function writeConfigs(payload) {
	const dir = path.join(os.homedir(), ".metassr", "runtime", "configurations");
	fs.mkdirSync(dir, { recursive: true });
	const nodeLoader = {
		search_paths: [],
		dependencies: { node: [libnodePath(payload)] },
		environment: [
			{ name: "NODE_PATH", value: path.join(payload, "lib", "node_modules") }
		]
	};
	const pythonPaths = [
		path.join(payload, "lib", "python3.14", "site-packages"),
		path.join(payload, "lib", "python3.14", "dist-packages")
	];
	const pyLoader = {
		search_paths: [],
		dependencies: { python: [libpythonPath(payload)] },
		environment: [
			{ name: "PYTHONHOME", value: payload },
			{ name: "PYTHONPATH", value: pythonPaths.join(path.delimiter) }
		]
	};
	const global = {
		node_loader: path.join(dir, "node_loader.json"),
		py_loader: path.join(dir, "py_loader.json"),
		log_level: "Error"
	};
	fs.writeFileSync(
		path.join(dir, "node_loader.json"),
		JSON.stringify(nodeLoader, null, "\t")
	);
	fs.writeFileSync(
		path.join(dir, "py_loader.json"),
		JSON.stringify(pyLoader, null, "\t")
	);
	fs.writeFileSync(path.join(dir, "global.json"), JSON.stringify(global, null, "\t"));
	return path.join(dir, "global.json");
}

function seedBundler(payload) {
	const source = path.join(payload, "vendor", "bundler");
	if (!fs.existsSync(source)) {
		return;
	}
	const target = path.join(os.homedir(), ".metassr", "vendor", "bundler");
	const marker = path.join(target, "node_modules", "esbuild");
	const packageJson = path.join(target, "package.json");
	if (fs.existsSync(marker) && fs.existsSync(packageJson)) {
		const bundled = fs.readFileSync(path.join(source, "package.json"), "utf8").trim();
		const installed = fs.readFileSync(packageJson, "utf8").trim();
		if (bundled === installed) {
			return;
		}
	}
	fs.rmSync(target, { recursive: true, force: true });
	fs.mkdirSync(path.dirname(target), { recursive: true });
	fs.cpSync(source, target, { recursive: true });
}

const payload = payloadDir();
const binary = path.join(payload, "bin", "metassr");
if (!fs.existsSync(binary)) {
	fail(`runtime payload is incomplete: ${binary} not found`);
}

seedBundler(payload);

const libraryPath = [
	path.join(payload, "node"),
	path.join(payload, "lib"),
	process.env.LD_LIBRARY_PATH
]
	.filter(Boolean)
	.join(path.delimiter);

const env = {
	...process.env,
	LD_LIBRARY_PATH: libraryPath,
	LOADER_LIBRARY_PATH: path.join(payload, "lib"),
	SERIAL_LIBRARY_PATH: path.join(payload, "lib"),
	CONFIGURATION_PATH: writeConfigs(payload)
};

try {
	execFileSync(binary, process.argv.slice(2), { stdio: "inherit", env });
} catch (error) {
	process.exit(typeof error.status === "number" ? error.status : 1);
}
