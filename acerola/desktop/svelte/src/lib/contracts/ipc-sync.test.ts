import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// Structural guard: ensures every Svelte `invoke()` points to a
// #[tauri::command] actually registered in bios::build(). Without it, renaming
// or removing a command in Rust only breaks at runtime.

const contractsDir = path.dirname(fileURLToPath(import.meta.url));
const svelteSrcDir = path.resolve(contractsDir, '../..');
const acerolaRoot = path.resolve(contractsDir, '../../../..');
const rustHandlerFile = path.join(acerolaRoot, 'src-tauri/src/bios/mod.rs');

function listFiles(dir: string, suffix: string): string[] {
	return readdirSync(dir, { recursive: true })
		.map((entry) => String(entry))
		.filter((entry) => entry.endsWith(suffix))
		.map((entry) => path.join(dir, entry));
}

function extractRustCommands(): Set<string> {
	const source = readFileSync(rustHandlerFile, 'utf-8');
	const handlerBlock = source.match(/invoke_handler\(tauri::generate_handler!\[([\s\S]*?)\]\)/);
	if (!handlerBlock) {
		throw new Error(`Could not find the invoke_handler block in ${rustHandlerFile}`);
	}

	const commands = new Set<string>();
	for (const match of handlerBlock[1].matchAll(/(\w+)::(\w+)/g)) {
		commands.add(match[2]);
	}
	return commands;
}

// Assumes "Const.prop" pairs are globally unique across the *.commands.ts files,
// which holds today even with duplicated const names (e.g. LIBRARY_COMMANDS
// in different modules), since their properties do not overlap.
function extractContractCommandMap(): Map<string, string> {
	const map = new Map<string, string>();

	for (const file of listFiles(contractsDir, '.commands.ts')) {
		const source = readFileSync(file, 'utf-8');
		for (const constMatch of source.matchAll(/export const (\w+) = \{([\s\S]*?)\} as const;/g)) {
			const [, constName, body] = constMatch;
			for (const propMatch of body.matchAll(/(\w+)\s*:\s*'([^']+)'/g)) {
				const [, propName, value] = propMatch;
				map.set(`${constName}.${propName}`, value);
			}
		}
	}

	return map;
}

interface InvokeCall {
	file: string;
	commandRef: string;
}

function findInvokeCalls(): InvokeCall[] {
	const files = [...listFiles(svelteSrcDir, '.ts'), ...listFiles(svelteSrcDir, '.svelte')].filter(
		(file) => !file.endsWith('.test.ts') && !file.includes(`${path.sep}paraglide${path.sep}`)
	);

	const calls: InvokeCall[] = [];
	for (const file of files) {
		const source = readFileSync(file, 'utf-8');
		for (const match of source.matchAll(
			/\binvoke(?:<[^()]*>)?\(\s*([A-Z][A-Z0-9_]*\.\w+|'[^']+')/g
		)) {
			calls.push({ file: path.relative(acerolaRoot, file), commandRef: match[1] });
		}
	}
	return calls;
}

describe('IPC contract sync (Svelte contracts vs. Rust commands)', () => {
	it('every static invoke() resolves to a command registered in bios::build()', () => {
		const rustCommands = extractRustCommands();
		const contractMap = extractContractCommandMap();
		const invokeCalls = findInvokeCalls();

		expect(rustCommands.size).toBeGreaterThan(0);
		expect(invokeCalls.length).toBeGreaterThan(0);

		const mismatches = invokeCalls.flatMap(({ file, commandRef }) => {
			const commandName = commandRef.startsWith("'")
				? commandRef.slice(1, -1)
				: contractMap.get(commandRef);

			if (commandName === undefined) {
				return [`${file}: unresolved reference "${commandRef}"`];
			}

			if (!rustCommands.has(commandName)) {
				return [
					`${file}: "${commandRef}" -> "${commandName}" without a matching #[tauri::command]`
				];
			}

			return [];
		});

		expect(mismatches).toEqual([]);
	});
});
