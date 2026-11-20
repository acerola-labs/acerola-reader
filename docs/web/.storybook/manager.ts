import { addons } from 'storybook/manager-api';
import { create } from 'storybook/theming';

addons.setConfig({
	theme: create({
		base: 'dark',
		brandTitle: 'Acerola Docs',
		// Brand logo from the brand CDN; the tab favicon is set in manager-head.html.
		brandImage: 'https://cdn.acerola.app/brand/logo/acerola-logo-foreground.svg',
		brandTarget: '_self'
	})
});

const style = document.createElement('style');
style.textContent = `
  .sidebar-header a img {
    max-height: 34px;
    width: auto;
  }
`;
document.head.appendChild(style);
