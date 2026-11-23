#!/usr/bin/env node
// Converts the acerola-brand mascot SVG (paths/circles with fill/stroke, groups with
// translate/scale) into an Android VectorDrawable. Covers only what that file uses and fails on
// anything else, so a brand change that needs more support is noticed.
//   node scripts/design/android-mascot.mjs <acerola-mascot.svg> acerola/android/ui/src/main/res/drawable/acerola_mascot.xml
// The svg comes from the brand (../acerola-brand/acerola-brand/static/brand/mascot/ or the CDN).
import { readFileSync, writeFileSync } from 'node:fs';

const [input, output] = process.argv.slice(2);
const svg = readFileSync(input, 'utf8');

const attrs = (tag) => Object.fromEntries([...tag.matchAll(/([\w:-]+)="([^"]*)"/g)].map((m) => [m[1], m[2]]));
const root = attrs(/<svg\b[^>]*>/.exec(svg)[0]);
const [vx, vy, vw, vh] = root.viewBox.split(/\s+/).map(Number);
const width = 240;
const height = Math.round((width * vh) / vw);

/** translate(a,b) scale(s) translate(c,d) → VectorDrawable group (pivot + scale + translate). */
function group(transform) {
	if (!transform) return '<group>';
	const ops = [...transform.matchAll(/(translate|scale)\(([^)]*)\)/g)].map((m) => [m[1], m[2].split(/[\s,]+/).map(Number)]);
	const shape = ops.map(([op]) => op).join(' ');
	if (shape === 'translate') return `<group android:translateX="${ops[0][1][0]}" android:translateY="${ops[0][1][1] ?? 0}">`;
	if (shape !== 'translate scale translate') throw new Error(`unsupported transform: ${transform}`);
	const [[, [ax, ay]], [, [s]], [, [cx, cy]]] = ops;
	// x' = s(x + c) + a = s(x - p) + p + t with pivot p = -c and t = a + c.
	return `<group android:pivotX="${-cx}" android:pivotY="${-cy}" android:scaleX="${s}" android:scaleY="${s}" android:translateX="${ax + cx}" android:translateY="${ay + cy}">`;
}

function shape(tag, name) {
	const a = attrs(tag);
	let d = a.d;
	if (name === 'circle') {
		const [cx, cy, r] = [a.cx, a.cy, a.r].map(Number);
		d = `M${cx - r},${cy}a${r},${r} 0 1,0 ${2 * r},0a${r},${r} 0 1,0 ${-2 * r},0Z`;
	}
	const out = [`android:pathData="${d}"`];
	if (a.fill && a.fill !== 'none') out.push(`android:fillColor="${a.fill.toUpperCase()}"`);
	if (a.stroke && a.stroke !== 'none') {
		out.push(`android:strokeColor="${a.stroke.toUpperCase()}"`);
		out.push(`android:strokeWidth="${a['stroke-width'] ?? 1}"`);
		out.push('android:strokeLineCap="round"');
		out.push(`android:strokeLineJoin="${a['stroke-linejoin'] ?? 'round'}"`);
	}
	for (const key of Object.keys(a)) {
		if (!['d', 'cx', 'cy', 'r', 'fill', 'stroke', 'stroke-width', 'stroke-linejoin', 'id'].includes(key))
			throw new Error(`unsupported attribute ${key} on ${name}`);
	}
	return `<path ${out.join(' ')}/>`;
}

const body = svg.slice(svg.indexOf('>', svg.indexOf('<svg')) + 1, svg.lastIndexOf('</svg>'));
const parts = [];
for (const match of body.matchAll(/<(\/?)(g|path|circle)\b([^>]*?)(\/?)>/g)) {
	const [tag, closing, name, , selfClosing] = match;
	if (closing) parts.push('</group>');
	else if (name === 'g') parts.push(group(attrs(tag).transform) + (selfClosing ? '</group>' : ''));
	else parts.push(shape(tag, name));
}
const unknown = body.replace(/<(\/?)(g|path|circle)\b[^>]*>/g, '').replace(/<!--[\s\S]*?-->/g, '').trim();
if (unknown) throw new Error(`unsupported content: ${unknown.slice(0, 80)}`);

const xml = `<?xml version="1.0" encoding="utf-8"?>
<!-- acerola-brand mascot (brand/mascot/acerola-mascot.svg) as a VectorDrawable. Never recolor,
     stretch, rotate or redraw it: change the brand file and convert again. -->
<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="${width}dp"
    android:height="${height}dp"
    android:viewportWidth="${vw}"
    android:viewportHeight="${vh}">
    <group android:translateX="${-vx}" android:translateY="${-vy}">
        ${parts.join('\n        ')}
    </group>
</vector>
`;
writeFileSync(output, xml);
console.log(`ok ${output} (${parts.length} elements, ${width}x${height}dp)`);
