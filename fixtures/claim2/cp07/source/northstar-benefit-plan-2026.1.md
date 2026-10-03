NORTHSTAR WORKFORCE BENEFIT PLAN
Outpatient Diagnostic Imaging Benefit Policy
Document ID: NS-WB-IMG-POLICY
Version: 2026.1
Effective date: 2026-01-01
Issued by: Northstar Synthetic Benefits Trust
Status: complete governing schedule for outpatient diagnostic imaging claims

1. PURPOSE, SCOPE, AND AUTHORITY

This schedule states the complete coverage and payment rules for outpatient
diagnostic imaging claims submitted under the Northstar Workforce Benefit
Plan, product NS-WB-2026. It governs eligibility, covered service classes,
network conditions, member cost sharing, the annual imaging benefit limit,
exclusions, adjudication order, notice, and review. It applies to services on
or after its effective date during the 2026 plan year. Monetary values in
claim records and decisions are United States dollars represented in integer
cents. This document is a synthetic benchmark policy and has no real issuer or
beneficiary.

Only this numbered version controls the synthetic claims that identify
NS-WB-IMG-POLICY version 2026.1. The version identifier is immutable for a
claim once assigned. A later plan publication does not change a frozen claim's
source revision or the rules applied to it.

2. DEFINITIONS

2.1 Allowed charge is the lower of the submitted charge and the plan's
contracted amount for the service, after ordinary coding and network edits.
The allowed charge is supplied as a verified case-source fact; adjudicators
must not substitute the billed charge.

2.2 Plan year is the calendar year containing the service date. Deductible,
prior benefit payments, and the annual imaging limit are measured separately
for each covered member and plan year.

2.3 In-network facility is a facility whose network status is active on the
service date in the frozen case source. A facility directory row alone does
not establish that the member or service is eligible.

2.4 Experimental service is an imaging method or device documented by the
claim source as not established for ordinary clinical use and not listed in
the covered service schedule. The exclusion in clause EXCL-8.4 applies only
when both conditions are present in the source evidence.

2.5 DEF-2.5 - Plan benefit payment is the amount payable by the plan after deductible
and coinsurance, limited by clause LIMIT-4.1. Member responsibility is the
allowed charge minus the final plan benefit payment. It includes deductible,
coinsurance, and any allowed charge left unpaid because of the annual limit.

3. MEMBER AND SERVICE ELIGIBILITY

3.1 ELIG-3.1 - A member is eligible for a service when enrollment is active on its date,
the identified product is NS-WB-2026, and the service falls inside the plan
year. Eligibility is based on the member and product facts in the unchanged
case source.

3.2 COV-3.2 - Covered outpatient diagnostic imaging includes standard computed
tomography, magnetic-resonance, and ultrasound services identified in the
published imaging schedule. Service code IMG-CT-04 is standard outpatient
computed tomography. A covered service must be performed at an in-network
facility and must be supported by an effective referral where clause 3.3
requires one.

3.3 COV-3.3 - IMG-CT-04 requires a referring-clinician order recorded before the service
date. The referral must identify the member, the imaging service class, and a
valid interval containing the service date. A referral is evidence of
eligibility; it does not set the allowed charge or change member cost sharing.

3.4 A service outside the published covered schedule is not covered under
this imaging benefit. An incomplete claim may be returned for records review;
it is not converted to a covered claim merely because a referral exists.

4. NETWORK AND CHARGE BASIS

4.1 NET-4.1 - The frozen case source supplies the network result for the service date.
Only the `in_network` value is eligible for this schedule's ordinary benefit
calculation. Out-of-network charges are not repriced under the in-network
terms in this document.

4.2 CHARGE-4.2 - The verified allowed charge is the basis for all calculations below. A
submitted charge greater than the allowed charge creates no additional plan
benefit. The plan does not round intermediate values because all rates and
amounts in this schedule produce whole cents for the claims admitted to this
synthetic workflow.

5. CLAIM RECORDS AND READ-ONLY VERIFICATION

5.1 The case-source snapshot is the controlling record for enrollment,
product, service date and code, facility, referral, allowed charge, remaining
deductible, annual imaging cap, prior cap payments, and exclusion facts.

5.2 Eligibility review and payment arithmetic are read-only operations. They
must cite the snapshot revision and must not alter enrollment, referral,
claims history, deductible balance, cap history, or plan text.

5.3 The eligibility receipt records the active-member, product, covered-code,
network, and referral findings. The benefit-check receipt records each
arithmetic input and result. A receipt is evidence of the check performed; it
does not create or revise the source fact that it verifies.

6. DEDUCTIBLE AND COINSURANCE

6.1 COST-6.1 - The annual individual imaging deductible is $1,500.00. For a claim, the
deductible amount applied is the lower of the allowed charge and the
member's remaining deductible immediately before the claim. The applied
amount is member responsibility. Deductible remaining after the claim equals
the prior remaining amount minus the amount applied.

6.2 COST-6.2 - After the deductible in clause 6.1, the member pays 20 percent of the
remaining allowed charge as coinsurance. The plan's pre-limit amount is the
other 80 percent. Compute both amounts from integer cents. The coinsurance
calculation does not include a second deductible and does not reset within a
claim.

6.3 There is no separate copayment for IMG-CT-04. If eligibility is absent,
the claim receives no benefit under this schedule and the cost-share
calculation is not used to imply coverage.

7. ANNUAL IMAGING BENEFIT LIMIT

7.1 LIMIT-7.1 - The maximum plan benefit payment for covered outpatient imaging is
$10,000.00 per member per plan year. Prior payment history is supplied as
plan payments already applied to this limit; it is not the member's allowed
charge history.

7.2 LIMIT-7.2 - Remaining limit equals the annual maximum minus prior applied plan
payments, floored at zero. The payable plan benefit is the lower of the
pre-limit plan amount under clause 6.2 and the remaining limit. A negative
remaining amount is treated as zero.

7.3 LIMIT-7.3 - The difference between the pre-limit plan amount and the payable benefit
is unpaid because the annual plan limit has been reached. Under clause 2.5,
that difference remains part of member responsibility. It must be reported
separately from deductible and coinsurance.

8. EXCLUSIONS

8.1 A claim must be covered under clauses 3 and 4 before payment terms are
applied. Exclusions are evaluated against the exact facts in the frozen case
source and the plan revision pinned to that case.

8.2 Services performed at an out-of-network facility are outside this
schedule's in-network benefit. This network restriction is evaluated under
clause 4.1.

8.3 A service without the referral required by clause 3.3 is not eligible for
this schedule. Missing referral evidence is an eligibility failure, not a
deductible or limit adjustment.

8.4 EXCL-8.4 - EXPERIMENTAL IMAGING METHOD OR DEVICE. This exclusion is
triggered only when the frozen claim source records both (a) an experimental
method or device and (b) absence from the published covered schedule. When
triggered, no plan benefit is payable for that service. When either condition
is false, this exclusion is not triggered. For IMG-CT-04 performed with the
standard listed method, a documented false experimental-method fact means
EXCL-8.4 is not triggered.

8.5 No other exclusion modifies the eligible standard service, allowed
charge, deductible, coinsurance, or annual limit calculations in this
schedule. The final decision must identify the applicable exclusion check
and its triggered status rather than infer one from a generic denial rule.

9. ADJUDICATION ORDER

9.1 ADJ-9.1 - Verify member, product, service date, code, facility, and referral under
clauses 3 and 4. Record a coverage status of COVERED only if each required
eligibility condition is met.

9.2 ADJ-9.2 - Evaluate the experimental-service condition in EXCL-8.4 using the frozen
source facts. If it is triggered, set the plan payment to zero and identify
the exclusion. Do not apply deductible, coinsurance, or annual-limit
calculations to an excluded service.

9.3 ADJ-9.3 - For an eligible non-excluded service, apply the remaining deductible to
the allowed charge, compute member and plan shares under clause 6, then apply
the remaining annual limit under clause 7. Keep the deductible, coinsurance,
limit adjustment, plan benefit, and total member responsibility as distinct
amounts.

9.4 ADJ-9.4 - A covered claim whose payable plan benefit is reduced by clause 7 is
adjudicated as COVERED_WITH_CAP_LIMIT. The reduced benefit does not change
the coverage status of the service.

10. NOTICE, REVIEW, AND RECORD RETENTION

10.1 The final notice identifies the policy version, covered service,
governing clause IDs, eligibility and exclusion result, the allowed charge,
each cost-share calculation, remaining limit, plan benefit, and member
responsibility. Amounts are reported in integer cents in the synthetic
decision record.

10.2 A member may request a clerical review within 60 calendar days of the
notice. Review checks source identity and arithmetic against the version
assigned to the claim. It does not mutate the source snapshot or silently
replace the governing plan version.

10.3 The adjudication packet retains its case-source revision, policy source
revision, check receipts, and final disposition together. Each final review
packet contains the complete controlling policy body as a native attachment
so the decision can be read with the authority applied to it.

11. VERSION CONTROL AND INTERPRETATION

11.1 Version 2026.1 is effective from 2026-01-01 through 2026-12-31 for this
synthetic product. Its clauses are applied as written to a case that pins
this version and remains unchanged during review.

11.2 Where an operational receipt reports an input, use the corresponding
case-source value. Where the case source and this schedule jointly determine
a result, follow the order in clause 9. A receipt cannot amend a clause, and
the evaluation answer cannot change source facts.

11.3 This document contains the complete terms needed to decide an
IMG-CT-04 outpatient imaging claim under NS-WB-2026: eligibility, covered
service, network, referral, allowed-charge basis, deductible, coinsurance,
annual plan limit, exclusion, adjudication, notice, review, and version
control. No external policy document or unstated rule is required for this
synthetic case.
