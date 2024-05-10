import { useLocation } from 'preact-iso';

export function Header() {
	const { url } = useLocation();

	return (
		<header>
			<nav>
				<a href="/" class={url == '/' && 'active'}>
					Home
				</a>
				<a href="/algorithms" class={url == '/algorithms' && 'active'}>
					Algorithms
				</a>
				<a href="/htop" class={url == '/htop' && 'active'}>
					Web-HTOP
				</a>
			</nav>
		</header>
	);
}
