type NotificationOptions = {
	action?: { label: string; onClick: () => void };
	description?: string;
	duration?: number;
};

type Notification<V extends string> = {
	message: string;
	variant: V;
	id: number;
} & NotificationOptions;

type NotifyMethods<V extends string> = {
	[K in V]: (message: string, options?: NotificationOptions) => number;
};

export function createNotifications<V extends string>(variants: readonly V[]) {
	let notifications = $state<Notification<V>[]>([]);
	let _id = 0;

	function add(message: string, options?: NotificationOptions & { variant: V }): number {
		const id = _id++;

		const notify = {
			id,
			message,
			duration: 5000,
			// Stryker disable next-line OptionalChaining,LogicalOperator: provably dead fallback —
			// add() is private, only called by notify.<variant>(...) (see line ~51), which ALWAYS
			// injects an explicit `variant` into options before calling add(). So `options` never
			// arrives undefined here, and the `...options` a few fields below unconditionally
			// overwrites this value anyway. Verified empirically by applying both mutants by
			// hand: the public tests (including a call with no options at all) pass the same way,
			// since the difference cannot be observed through the exported API. See
			// notification.test.ts.
			variant: options?.variant ?? variants[0],
			...options
		};

		notifications.push(notify);
		if (notify.duration > 0) setTimeout(() => pop(id), notify.duration);

		return id;
	}

	function pop(id: number) {
		const index = notifications.findIndex((it) => it.id === id);
		if (index !== -1) notifications.splice(index, 1);
	}

	function clearAll() {
		notifications.splice(0, notifications.length);
	}

	// derives notify.* from the keys passed at bootstrap
	const notify = Object.fromEntries(
		variants.map((variant) => [
			variant,
			(message: string, options?: NotificationOptions) =>
				add(message, { ...options, variant })
		])
	) as NotifyMethods<V>;

	return {
		pop,
		notify,
		clearAll,
		get notifications() {
			return notifications;
		}
	};
}
