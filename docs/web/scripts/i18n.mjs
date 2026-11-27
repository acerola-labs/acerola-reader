/**
 * Compiles the messages (Paraglide) into src/lib/paraglide/ with the options in
 * paraglide.config.ts: the CLI cannot take urlPatterns.
 */
import { compile } from '@inlang/paraglide-js';
import { paraglide } from '../paraglide.config.ts';

await compile(paraglide);
