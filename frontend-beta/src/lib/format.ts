function toValidDate(value: string | number | Date): Date {
	const date = new Date(value);
	if (Number.isNaN(date.getTime())) {
		return new Date(0);
	}
	return date;
}

function pad2(value: number): string {
	return String(value).padStart(2, "0");
}

export function formatUtcTime(value: string | number | Date): string {
	const date = toValidDate(value);
	return `${pad2(date.getUTCHours())}:${pad2(date.getUTCMinutes())}:${pad2(date.getUTCSeconds())}`;
}

export function formatUtcDateTime(value: string | number | Date): string {
	const date = toValidDate(value);
	return `${date.getUTCFullYear()}-${pad2(date.getUTCMonth() + 1)}-${pad2(date.getUTCDate())} ${formatUtcTime(date)} UTC`;
}

export function stableRotationFromId(id: string, range = 30): number {
	let hash = 0;
	for (let i = 0; i < id.length; i++) {
		hash = (hash << 5) - hash + id.charCodeAt(i);
		hash |= 0;
	}

	const span = range * 2 + 1;
	const normalized = ((hash % span) + span) % span;
	return normalized - range;
}
