"use client";

import React, { useEffect, useMemo, useState } from "react";

type CardState = {
	number: string;
	holder: string;
	month: string;
	year: string;
	cvv: string;
};

type CardValidity = {
	number: boolean;
	holder: boolean;
	month: boolean;
	year: boolean;
	cvv: boolean;
	allValid: boolean;
};

type Props = {
	defaultNumber?: string;
	defaultHolder?: string;
	defaultMonth?: string;
	defaultYear?: string;
	defaultCVV?: string;
	maskMiddle?: boolean;
	ring1?: string;
	ring2?: string;
	showSubmit?: boolean;
	onChange?: (state: CardState, validity: CardValidity) => void;
	onSubmit?: (state: CardState, validity: CardValidity) => void;
	className?: string;
};

function formatNumberSpaces(num: string): string {
	return num.replace(/\s+/g, "").replace(/(\d{4})(?=\d)/g, "$1 ");
}

function clampDigits(value: string, maxLen: number) {
	return value.replace(/\D/g, "").slice(0, maxLen);
}

const CreditCardForm = ({
	defaultNumber = "",
	defaultHolder = "",
	defaultMonth = "",
	defaultYear = "",
	defaultCVV = "",
	maskMiddle = true,
	ring1 = "rgba(120, 145, 255, 0.85)",
	ring2 = "rgba(220, 90, 200, 0.7)",
	showSubmit = true,
	onChange,
	onSubmit,
	className = "",
}: Props) => {
	const [number, setNumber] = useState(clampDigits(defaultNumber, 19));
	const [holder, setHolder] = useState(defaultHolder.toUpperCase());
	const [month, setMonth] = useState(defaultMonth);
	const [year, setYear] = useState(defaultYear);
	const [cvv, setCVV] = useState(clampDigits(defaultCVV, 4));
	const [focusField, setFocusField] = useState<
		null | "number" | "holder" | "expire" | "cvv"
	>(null);

	const flip = focusField === "cvv";
	const years = useMemo(() => {
		const start = new Date().getFullYear();
		return Array.from({ length: 10 }, (_, i) => String(start + i));
	}, []);

	const validity: CardValidity = useMemo(() => {
		const numberValid = number.length >= 13;
		const holderValid = holder.trim().length >= 2;
		const monthValid = !!month && +month >= 1 && +month <= 12;
		const yearValid = !!year && +year >= new Date().getFullYear();
		const cvvValid = /^\d{3,4}$/.test(cvv);
		return {
			number: numberValid,
			holder: holderValid,
			month: monthValid,
			year: yearValid,
			cvv: cvvValid,
			allValid:
				numberValid && holderValid && monthValid && yearValid && cvvValid,
		};
	}, [number, holder, month, year, cvv]);

	useEffect(() => {
		onChange?.({ number, holder, month, year, cvv }, validity);
	}, [number, holder, month, year, cvv, validity, onChange]);

	const displayDigits = useMemo(() => number.slice(0, 16).split(""), [number]);

	const displayedSlots = useMemo(() => {
		const arr: { textTop: string; filed: boolean }[] = [];
		for (let i = 0; i < 16; i++) {
			let content = "•";
			if (i < displayDigits.length) {
				const d = displayDigits[i];
				const shouldMask = maskMiddle && i >= 4 && i <= 11;
				content = shouldMask ? "*" : d;
			}
			arr.push({ textTop: content, filed: i < displayDigits.length });
		}
		return arr;
	}, [displayDigits, maskMiddle]);

	const handleSubmit = (e: React.FormEvent) => {
		e.preventDefault();
		onSubmit?.({ number, holder, month, year, cvv }, validity);
	};

	return (
		<section className={`ccp ${className}`}>
			<div className="wrap">
				{/* CARD */}
				<section className={`card ${flip ? "flip" : ""}`}>
					{/* FRONT */}
					<section
						className="card__front"
						style={{
							["--ring1" as string]: ring1,
							["--ring2" as string]: ring2,
						}}
					>
						<div className="card__header">
							<div className="card__brand">
								<span className="card__brand-mark">D</span>
								<span>DeepMail</span>
							</div>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								height="32"
								width="48"
								viewBox="-96 -98.908 832 593.448"
							>
								<path fill="#ff5f00" d="M224.833 42.298h190.416v311.005H224.833z" />
								<path
									d="M244.446 197.828a197.448 197.448 0 0175.54-155.475 197.777 197.777 0 100 311.004 197.448 197.448 0 01-75.54-155.53z"
									fill="#eb001b"
								/>
								<path
									d="M640 197.828a197.777 197.777 0 01-320.015 155.474 197.777 197.777 0 000-311.004A197.777 197.777 0 01640 197.773z"
									fill="#f79e1b"
								/>
							</svg>
						</div>

						<div className="card__number" aria-label="Card number">
							{displayedSlots.map((slot, idx) => (
								<span key={idx} className="slot">
									<span className={`digit ${slot.filed ? "filed" : ""}`}>
										<span className="row placeholder">•</span>
										<span className="row value">{slot.textTop}</span>
									</span>
								</span>
							))}
						</div>

						<div className="card__footer">
							<div className="card__holder">
								<div className="card__section__title">Card Holder</div>
								<div>{holder || "NAME ON CARD"}</div>
							</div>
							<div className="card__expires">
								<div className="card__section__title">Expires</div>
								<span>{month || "MM"}</span>/
								<span>{year ? year.slice(-2) : "YY"}</span>
							</div>
						</div>
					</section>

					{/* BACK */}
					<section
						className="card__back"
						style={{
							["--ring1" as string]: ring1,
							["--ring2" as string]: ring2,
						}}
					>
						<div className="card__hide_line" />
						<div className="card_cvv">
							<span>CVV</span>
							<div className="card_cvv_field">
								{cvv ? "*".repeat(cvv.length) : "***"}
							</div>
						</div>
					</section>
				</section>

				{/* FORM */}
				<form className="form" onSubmit={handleSubmit} noValidate>
					<div>
						<label htmlFor="number">Card Number</label>
						<input
							id="number"
							inputMode="numeric"
							autoComplete="cc-number"
							placeholder="1234 5678 9012 3456"
							value={formatNumberSpaces(number)}
							onChange={(e) => setNumber(clampDigits(e.target.value, 19))}
							onFocus={() => setFocusField("number")}
							onBlur={() => setFocusField(null)}
							aria-invalid={!validity.number && number.length >= 13}
						/>
						{!validity.number && number.length >= 13 && (
							<small className="err">Card number looks invalid</small>
						)}
					</div>

					<div>
						<label htmlFor="holder">Card Holder</label>
						<input
							id="holder"
							type="text"
							autoComplete="cc-name"
							placeholder="JANE DOE"
							value={holder}
							onChange={(e) => setHolder(e.target.value.toUpperCase())}
							onFocus={() => setFocusField("holder")}
							onBlur={() => setFocusField(null)}
						/>
					</div>

					<div className="filed__group">
						<div>
							<label>Expiration Date</label>
							<div className="filed__date">
								<select
									value={month || ""}
									onChange={(e) => setMonth(e.target.value)}
									onFocus={() => setFocusField("expire")}
									onBlur={() => setFocusField(null)}
								>
									<option value="" disabled>
										Month
									</option>
									{Array.from({ length: 12 }, (_, i) =>
										String(i + 1).padStart(2, "0"),
									).map((m) => (
										<option key={m} value={m}>
											{m}
										</option>
									))}
								</select>
								<select
									value={year || ""}
									onChange={(e) => setYear(e.target.value)}
									onFocus={() => setFocusField("expire")}
									onBlur={() => setFocusField(null)}
								>
									<option value="" disabled>
										Year
									</option>
									{years.map((y) => (
										<option key={y} value={y}>
											{y}
										</option>
									))}
								</select>
							</div>
						</div>

						<div>
							<label htmlFor="cvv">CVV</label>
							<input
								id="cvv"
								inputMode="numeric"
								autoComplete="cc-csc"
								placeholder="•••"
								value={cvv}
								onChange={(e) => setCVV(clampDigits(e.target.value, 4))}
								onFocus={() => setFocusField("cvv")}
								onBlur={() => setFocusField(null)}
							/>
						</div>
					</div>

					{showSubmit && (
						<button
							className="submit"
							type="submit"
							disabled={!validity.allValid}
							aria-disabled={!validity.allValid}
						>
							{validity.allValid ? "Save payment method" : "Complete all fields"}
						</button>
					)}
				</form>
			</div>

			<style jsx>{`
				.ccp {
					width: 100%;
					display: flex;
					justify-content: center;
					color: rgba(255, 255, 255, 0.85);
				}
				.wrap {
					width: 100%;
					max-width: 1000px;
					display: grid;
					grid-template-columns: 1fr 1fr;
					gap: 32px;
					align-items: start;
				}

				@media (max-width: 920px) {
					.wrap {
						grid-template-columns: 1fr;
					}
				}

				* {
					box-sizing: border-box;
				}

				.card {
					position: relative;
					width: 100%;
					max-width: 440px;
					margin: 0 auto;
					transform-style: preserve-3d;
					transition: 0.8s;
					perspective: 1000px;
				}
				.card.flip {
					transform: rotateY(180deg);
				}

				.card__front,
				.card__back {
					width: 100%;
					max-width: 440px;
					height: 248px;
					border-radius: 24px;
					padding: 24px 30px 30px;
					background:
						linear-gradient(135deg, rgba(28, 30, 38, 0.95), rgba(8, 10, 14, 0.95)),
						radial-gradient(circle at 0% 0%, rgba(120, 145, 255, 0.18), transparent 60%);
					box-shadow:
						0 24px 60px rgba(0, 0, 0, 0.5),
						0 0 0 1px rgba(255, 255, 255, 0.06);
					color: rgba(255, 255, 255, 0.92);
					overflow: hidden;
					margin: 0 auto;
					backface-visibility: hidden;
					position: relative;
				}

				@media (max-width: 450px) {
					.card__front,
					.card__back {
						padding: 14px 16px 18px;
						height: 220px;
					}
				}

				.card__back {
					position: absolute;
					top: 0;
					left: 0;
					transform: rotateY(180deg);
					padding: 24px 0 0;
				}

				.card__front::before,
				.card__back::before {
					content: "";
					position: absolute;
					border: 16px solid var(--ring1);
					border-radius: 100%;
					left: -17%;
					top: -45px;
					height: 280px;
					width: 280px;
					filter: blur(28px);
					opacity: 0.55;
				}

				.card__front::after,
				.card__back::after {
					content: "";
					position: absolute;
					border: 16px solid var(--ring2);
					border-radius: 100%;
					width: 280px;
					top: 55%;
					left: -200px;
					height: 280px;
					filter: blur(28px);
					opacity: 0.45;
				}

				.card__hide_line {
					height: 40px;
					width: 100%;
					background-color: rgba(255, 255, 255, 0.08);
					position: relative;
					z-index: 1;
				}

				.card_cvv {
					position: relative;
					z-index: 1;
					margin-top: 24px;
					padding: 0 32px;
					display: flex;
					flex-direction: column;
					align-items: end;
					font-size: 12px;
					font-weight: 600;
					text-transform: uppercase;
					color: rgba(255, 255, 255, 0.5);
					letter-spacing: 0.1em;
				}
				.card_cvv_field {
					margin-top: 8px;
					background-color: rgba(255, 255, 255, 0.92);
					border-radius: 12px;
					height: 44px;
					width: 100%;
					color: #000;
					display: flex;
					align-items: center;
					justify-content: end;
					padding: 0 14px;
					font-size: 22px;
					line-height: 21px;
					letter-spacing: 0.15em;
				}

				.card__header {
					display: flex;
					align-items: center;
					justify-content: space-between;
					font-weight: 600;
					margin-bottom: 32px;
					position: relative;
					z-index: 1;
				}
				.card__brand {
					display: flex;
					align-items: center;
					gap: 8px;
					font-size: 14px;
					letter-spacing: 0.02em;
				}
				.card__brand-mark {
					display: inline-flex;
					align-items: center;
					justify-content: center;
					height: 26px;
					width: 26px;
					border-radius: 8px;
					background: rgba(255, 255, 255, 0.06);
					border: 1px solid rgba(255, 255, 255, 0.1);
					font-size: 13px;
					font-weight: 700;
				}

				.card__number {
					font-size: 22px;
					margin-bottom: 32px;
					position: relative;
					z-index: 1;
					display: flex;
					height: 33px;
					overflow: hidden;
					color: rgba(255, 255, 255, 0.92);
					letter-spacing: 0.06em;
					font-variant-numeric: tabular-nums;
				}

				.card__number .slot {
					display: inline-flex;
					margin-right: 0;
				}

				.card__number .slot:nth-child(4n) {
					margin-right: 12px;
				}

				.card__number .digit {
					display: flex;
					flex-direction: column;
					height: 33px;
					line-height: 33px;
					transition: transform 0.2s;
				}

				.card__number .digit.filed {
					transform: translateY(-33px);
				}

				.card__number .row {
					height: 33px;
					display: block;
				}
				.card__number .placeholder {
					color: rgba(255, 255, 255, 0.25);
				}

				.card__footer {
					display: flex;
					align-items: center;
					justify-content: space-between;
					position: relative;
					z-index: 1;
					font-size: 14px;
				}
				.card__holder {
					text-transform: uppercase;
				}
				.card__section__title {
					font-size: 10px;
					font-weight: 600;
					text-transform: uppercase;
					letter-spacing: 0.18em;
					color: rgba(255, 255, 255, 0.4);
					margin-bottom: 4px;
				}

				.form {
					border-radius: 24px;
					background: rgba(255, 255, 255, 0.02);
					width: 100%;
					max-width: 600px;
					margin: 0 auto;
					padding: 28px;
					border: 1px solid rgba(255, 255, 255, 0.08);
					box-shadow: 0 24px 60px rgba(0, 0, 0, 0.35);
					display: grid;
					gap: 14px;
				}

				label {
					display: block;
					margin: 4px 0 6px;
					color: rgba(255, 255, 255, 0.45);
					font-weight: 500;
					font-size: 11px;
					text-transform: uppercase;
					letter-spacing: 0.15em;
				}

				input,
				select {
					height: 44px;
					display: block;
					width: 100%;
					border: 1px solid rgba(255, 255, 255, 0.1);
					padding: 0 16px;
					transition:
						border-color 200ms ease,
						background 200ms ease,
						box-shadow 200ms ease;
					border-radius: 16px;
					outline: none;
					background-color: rgba(255, 255, 255, 0.02);
					color: rgba(255, 255, 255, 0.92);
					font-size: 14px;
				}

				input::placeholder {
					color: rgba(255, 255, 255, 0.25);
				}

				input:hover,
				select:hover {
					border-color: rgba(255, 255, 255, 0.2);
				}

				input:focus,
				select:focus {
					border: 1px solid rgba(255, 255, 255, 0.3);
					background-color: rgba(255, 255, 255, 0.05);
					box-shadow: 0 0 0 4px rgba(255, 255, 255, 0.06);
				}

				input[aria-invalid="true"] {
					border-color: rgba(244, 63, 94, 0.5);
				}

				select {
					padding: 0 14px;
					appearance: none;
					background-image: linear-gradient(45deg, transparent 50%, rgba(255, 255, 255, 0.5) 50%),
						linear-gradient(135deg, rgba(255, 255, 255, 0.5) 50%, transparent 50%);
					background-position: calc(100% - 18px) center, calc(100% - 13px) center;
					background-size: 5px 5px;
					background-repeat: no-repeat;
				}

				.filed__group {
					display: grid;
					grid-template-columns: 2fr 1fr;
					gap: 18px;
				}

				@media (max-width: 560px) {
					.filed__group {
						grid-template-columns: 1fr;
					}
				}

				.filed__date {
					display: grid;
					grid-template-columns: 1fr 1fr;
					gap: 10px;
				}

				.err {
					color: rgba(244, 63, 94, 0.85);
					font-size: 11px;
					margin-top: 6px;
					display: block;
				}

				.submit {
					margin-top: 8px;
					height: 48px;
					border: 1px solid rgba(255, 255, 255, 0.2);
					border-radius: 999px;
					background:
						linear-gradient(180deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.04));
					color: rgba(255, 255, 255, 0.95);
					font-weight: 500;
					font-size: 14px;
					cursor: pointer;
					transition:
						border-color 200ms ease,
						background 200ms ease,
						box-shadow 200ms ease,
						transform 200ms ease;
				}
				.submit:not(:disabled):hover {
					border-color: rgba(255, 255, 255, 0.35);
					background: rgba(255, 255, 255, 0.1);
					box-shadow: 0 0 32px rgba(255, 255, 255, 0.16);
				}
				.submit:not(:disabled):active {
					transform: scale(0.99);
				}
				.submit:disabled {
					opacity: 0.5;
					cursor: not-allowed;
				}
			`}</style>
		</section>
	);
};

export { CreditCardForm };
export type { CardState, CardValidity };
