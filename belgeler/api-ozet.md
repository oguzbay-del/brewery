# API uç nokta özeti

Bu dosya sozlesme/openapi/openapi.json kaynağından belgeler/uret-api-ozet.py ile üretilir. Kaynak sözleşmeyi değiştirin; bu dosyayı elle düzenlemeyin.

Toplam **222** operasyon. Yetki ve adım bilgisi yalnız OpenAPI operasyon alanlarından alınır. Boş hücre, sözleşmede ilgili bilgi bulunmadığını gösterir.

## access

Roles, members, assignments, invitations and sharing

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| POST | `/v1/invitations/accept` | Accepts an invitation as the signed-in user: joins the workspace with the invited role. An unknown, used, revoked or expired token is `404`. |  |  |  |
| GET | `/v1/workspaces/{workspace_id}/invitations` | Lists pending invitations. | users.manage |  |  |
| POST | `/v1/workspaces/{workspace_id}/invitations` | Invites someone by mail. The link works once, for 7 days. | users.manage | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/invitations/{id}` | Revokes a pending invitation. | users.manage |  |  |
| GET | `/v1/workspaces/{workspace_id}/members` | Lists the members with their role assignments. | users.manage |  |  |
| DELETE | `/v1/workspaces/{workspace_id}/members/{id}` | Removes a member from the workspace together with their assignments. | users.manage |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/members/{id}` | Disables or re-enables a member. A disabled member loses access at once. | users.manage |  |  |
| POST | `/v1/workspaces/{workspace_id}/members/{id}/assignments` | Gives a member a role, for the whole workspace or one recipe, batch, vessel or area. | users.manage | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/members/{id}/assignments/{assignment_id}` | Removes a role assignment. | users.manage |  |  |
| GET | `/v1/workspaces/{workspace_id}/roles` | Lists the workspace's roles. | users.manage |  |  |
| POST | `/v1/workspaces/{workspace_id}/roles` | Creates a role from scratch. | users.manage | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/roles/{id}` | Deletes a role nobody holds; its pending invitations are revoked. `If-Match` is required. | users.manage |  | required |
| PATCH | `/v1/workspaces/{workspace_id}/roles/{id}` | Changes a role's name, landing screen or permissions. `If-Match` is required. | users.manage |  | required |
| POST | `/v1/workspaces/{workspace_id}/roles/{id}/copy` | Copies a role (preset or custom) under a new name. | users.manage | required |  |
| POST | `/v1/workspaces/{workspace_id}/shares` | Shares one recipe or batch with a friend (Home workspaces): a scoped invitation with the editor or viewer preset. | users.manage | required |  |

## account deletion

Account deletion and restore during the grace period

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| DELETE | `/v1/me/deletion` | Cancels a pending request while its grace period remains open. |  |  |  |
| GET | `/v1/me/deletion` | Current deletion request. A null result means there is no request. |  |  |  |
| POST | `/v1/me/deletion` | Requests permanent account deletion after the configured grace period. |  |  |  |

## auth

Sign-in with Apple, Google or an e-mail link

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| POST | `/v1/auth/apple` | Signs in with an Apple ID token. |  |  |  |
| POST | `/v1/auth/email/start` | Mails a sign-in link. Always `202`; whether the address is known never shows. |  |  |  |
| POST | `/v1/auth/email/verify` | Exchanges the mailed token for a session. The token works once. |  |  |  |
| POST | `/v1/auth/google` | Signs in with a Google ID token. |  |  |  |
| POST | `/v1/auth/nonce` | Opens a sign-in attempt: a single-use nonce (ADR-0009 rule 1.2). |  |  |  |

## batches

Batches, steps, actual amounts, measurements, the brew day package and the calendar

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/workspaces/{workspace_id}/batches` | Lists the batches of the workspace. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/batches` | Starts a batch from an approved recipe version and writes its steps. | batch.plan | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/batches/{id}` | Deletes a batch (soft delete). | batch.plan |  | required |
| GET | `/v1/workspaces/{workspace_id}/batches/{id}` | Reads a batch. | recipe.view |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/batches/{id}` | Changes the name, plan or volume of a batch. A closed batch answers `409`. | batch.plan |  | required |
| GET | `/v1/workspaces/{workspace_id}/batches/{id}/actuals` | Lists the actual amounts of a batch with plan and deviation. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/batches/{id}/actuals` | Records an ingredient that is not in the recipe. | batch.record | required |  |
| PUT | `/v1/workspaces/{workspace_id}/batches/{id}/actuals/{line_kind}/{line_id}` | Records the actual amount of a recipe line (create or replace). A deviation of 5 percent or more opens a deviation record (`ingredient_amount`). | batch.record |  |  |
| GET | `/v1/workspaces/{workspace_id}/batches/{id}/brew-day-package` | Reads the newest package of a batch. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/batches/{id}/brew-day-package` | Creates a signed brew day package for a batch. | batch.plan | required |  |
| GET | `/v1/workspaces/{workspace_id}/batches/{id}/measurements` | Lists the measurements of a batch in time order. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/batches/{id}/measurements` | Sends measurements. An item with a known `id` is not stored again, so the client can repeat the request after a lost answer. An item that names a package is checked against the package. | batch.record |  |  |
| GET | `/v1/workspaces/{workspace_id}/batches/{id}/steps` | Lists the steps of a batch in order. | recipe.view |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/batches/{id}/steps/{step_id}` | Records a step: start, end, status and input source. The newest write wins for each field. | batch.record |  | optional |
| GET | `/v1/workspaces/{workspace_id}/batches/{id}/summary` | Plan against actual for a batch: gravities, volume, alcohol, ingredient deviations, step times and the stage history. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/batches/{id}/transitions` | Moves a batch to its next stage, or cancels it. Other moves answer `409`. | batch.record |  | required |
| GET | `/v1/workspaces/{workspace_id}/brew-day-packages/{id}` | Reads one package. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/brew-day-packages/{id}/revoke` | Revokes a package. The server then rejects measurements that name it. The call is safe to repeat. | batch.plan |  |  |
| GET | `/v1/workspaces/{workspace_id}/calendar` | The calendar: batches with a planned brew day in a date range, and their steps. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/calendar/backward-plan` | Plans backward from the day the beer is ready: brew day, end of the fermentation and of the conditioning. The durations come from the fermentation profile of the recipe version. | recipe.view |  |  |

## devices

Devices: registration, pairing and the data address key

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/workspaces/{workspace_id}/devices` | Lists the devices of the workspace. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/devices` | Registers a device and returns its key once. The key is part of the data address. | devices.manage |  |  |
| DELETE | `/v1/workspaces/{workspace_id}/devices/{id}` | Deletes a device (soft delete). Its data address stops working at once. | devices.manage |  |  |
| GET | `/v1/workspaces/{workspace_id}/devices/{id}` | Reads a device. | recipe.view |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/devices/{id}` | Changes the name, the calibration or the paired batch (`null` unpairs). `If-Match` is optional: each reading also changes the version. | devices.manage |  |  |
| POST | `/v1/workspaces/{workspace_id}/devices/{id}/rotate-key` | Makes a new key and returns it once. The old key stops working at once. Needs the second step when the auth services run. | devices.manage |  |  |

## events

Live updates (SSE)

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/workspaces/{ws}/events` | Live updates of one workspace (SSE). |  |  |  |

## files

Attachments and scanned documents (direct upload to the object store)

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/workspaces/{workspace_id}/attachments` | Lists completed attachments. |  |  |  |
| POST | `/v1/workspaces/{workspace_id}/attachments` | Creates an attachment record and returns the upload instructions. | batch.record | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/attachments/{id}` | Deletes an attachment (soft delete). | batch.record |  |  |
| GET | `/v1/workspaces/{workspace_id}/attachments/{id}` | Reads one attachment. |  |  |  |
| POST | `/v1/workspaces/{workspace_id}/attachments/{id}/complete` | Confirms that the bytes were uploaded; the stored object is checked first. | batch.record |  |  |
| GET | `/v1/workspaces/{workspace_id}/attachments/{id}/download` | Gives a short lived link to download the file. |  |  |  |
| GET | `/v1/workspaces/{workspace_id}/scanned-documents` | Lists completed scanned documents. |  |  |  |
| POST | `/v1/workspaces/{workspace_id}/scanned-documents` | Creates a scanned document record and returns the upload instructions. | batch.record | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/scanned-documents/{id}` | Deletes a scanned document (soft delete); refused under legal hold. | batch.record |  |  |
| GET | `/v1/workspaces/{workspace_id}/scanned-documents/{id}` | Reads one scanned document. |  |  |  |
| POST | `/v1/workspaces/{workspace_id}/scanned-documents/{id}/complete` | Confirms that the bytes were uploaded; the stored object is checked first. | batch.record |  |  |
| GET | `/v1/workspaces/{workspace_id}/scanned-documents/{id}/download` | Gives a short lived link to download the file. |  |  |  |

## import-export

Recipe import (BeerXML, BeerJSON, Brewfather) and data export jobs

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| POST | `/v1/workspaces/{workspace_id}/exports` | Queues an export. The full export (`scope` `workspace`) runs once in 24 hours for a user. | reports.export |  |  |
| GET | `/v1/workspaces/{workspace_id}/exports/{id}` | Reads an export job. A finished job carries a download link with its expiry time. | reports.export |  |  |
| POST | `/v1/workspaces/{workspace_id}/imports` | Uploads a recipe file and parses it (nothing is created yet). | recipe.edit |  |  |
| GET | `/v1/workspaces/{workspace_id}/imports/{id}` | Reads an uploaded import file. | recipe.edit |  |  |
| POST | `/v1/workspaces/{workspace_id}/imports/{id}/apply` | Creates the recipes of an uploaded file, with the mapping and the calculated values. | recipe.edit |  |  |

## library

Ingredient library, style guides, styles and mappings between guides

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/workspaces/{workspace_id}/ingredients` | Lists the ingredients of the shared library and of the workspace. |  |  |  |
| POST | `/v1/workspaces/{workspace_id}/ingredients` | Adds an ingredient to the workspace. | recipe.edit | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/ingredients/{id}` | Deletes an ingredient of the workspace (soft delete). A shared library ingredient answers `403`. | recipe.edit |  | required |
| GET | `/v1/workspaces/{workspace_id}/ingredients/{id}` | Reads one ingredient with its detail. |  |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/ingredients/{id}` | Changes an ingredient of the workspace. A shared library ingredient answers `403`. | recipe.edit |  | required |
| GET | `/v1/workspaces/{workspace_id}/style-guides` | Lists the style guides. |  |  |  |
| GET | `/v1/workspaces/{workspace_id}/style-guides/{id}` | Reads one style guide. |  |  |  |
| GET | `/v1/workspaces/{workspace_id}/style-guides/{id}/styles` | Lists the styles of one guide. |  |  |  |
| GET | `/v1/workspaces/{workspace_id}/style-mappings` | Lists the mappings between the styles of two guides. |  |  |  |
| GET | `/v1/workspaces/{workspace_id}/styles` | Lists styles of all guides. |  |  |  |
| GET | `/v1/workspaces/{workspace_id}/styles/{id}` | Reads one style. |  |  |  |

## me

The signed-in user: session, sessions, sign-in methods

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/me/identities` | The user's sign-in methods. |  |  |  |
| POST | `/v1/me/identities` | Links another sign-in method to the account. The session is replaced. |  |  |  |
| DELETE | `/v1/me/identities/{id}` | Removes a sign-in method. The last one stays; a sign-in in the last 10 minutes is needed. |  |  |  |
| GET | `/v1/me/passkeys` | Lists the user's passkeys. |  |  |  |
| DELETE | `/v1/me/passkeys/{id}` | Removes a passkey. Needs the second step. | step-up |  |  |
| POST | `/v1/me/passkeys/registration/finish` | Finishes a passkey registration. The new session carries the second step. | step-up |  |  |
| POST | `/v1/me/passkeys/registration/start` | Starts a passkey registration. Needs the second step (a passkey check when the user already has one, otherwise a sign-in in the last 10 minutes). | step-up |  |  |
| DELETE | `/v1/me/session` | Signs out of the current session. |  |  |  |
| GET | `/v1/me/session` | The current session, the user and the user's workspaces. |  |  |  |
| DELETE | `/v1/me/sessions` | Signs out everywhere, this session included. |  |  |  |
| GET | `/v1/me/sessions` | The user's open sessions. |  |  |  |
| DELETE | `/v1/me/sessions/{id}` | Closes one of the user's sessions. |  |  |  |
| POST | `/v1/me/step-up/finish` | Finishes the second step: writes `step_up_at` and replaces the session token. | step-up |  |  |
| POST | `/v1/me/step-up/start` | Starts the second step with a passkey. | step-up |  |  |

## meta

Definitions and client requirements

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/meta/client-requirements` | Lowest supported client versions; readable before sign-in (ADR-0008 rule 8.3). |  |  |  |
| GET | `/v1/meta/fields` | Field definitions: limits, units, steps, modes and texts, from `common::catalog` (the one source the server validation also reads; ADR-0008 rule 7). |  |  |  |

## notifications

Notification inbox, preferences and alert rules

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/workspaces/{workspace_id}/alert-rules` | The alert rules of the workspace. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/alert-rules` | Creates an alert rule. | batch.record | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/alert-rules/{id}` | Deletes an alert rule. | batch.record |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/alert-rules/{id}` | Changes an alert rule. | batch.record |  |  |
| GET | `/v1/workspaces/{workspace_id}/me/notification-preferences` | The notification preferences of the caller. Defaults when nothing is saved. |  |  |  |
| PUT | `/v1/workspaces/{workspace_id}/me/notification-preferences` | Changes the notification preferences of the caller. |  |  |  |
| GET | `/v1/workspaces/{workspace_id}/notifications` | The inbox of the caller, newest first. |  |  |  |
| POST | `/v1/workspaces/{workspace_id}/notifications/{id}/actions` | Approves, postpones or ignores a notification. |  |  |  |

## recipes

Recipes, versions, folders, tags, favorites and the edit session

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/workspaces/{workspace_id}/folders` | Lists the folders of the workspace. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/folders` | Creates a folder. | recipe.edit | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/folders/{id}` | Deletes an empty folder (soft delete). A folder with subfolders or recipes answers `409`. | recipe.edit |  | optional |
| PATCH | `/v1/workspaces/{workspace_id}/folders/{id}` | Renames or moves a folder. | recipe.edit |  | optional |
| GET | `/v1/workspaces/{workspace_id}/recipe-versions/{id}` | Reads a version with its lines, snapshots and (when published) the calculation summary. | recipe.view |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/recipe-versions/{id}` | Changes a draft. Arrays replace the lines of their section. A stale `If-Match` answers `412` with the current version and `conflicting_fields` (the fields of the patch that differ). | recipe.edit |  | required |
| POST | `/v1/workspaces/{workspace_id}/recipe-versions/{id}/calculate` | Calculates a version with the engine. A published version returns its stored summary. | recipe.view |  |  |
| GET | `/v1/workspaces/{workspace_id}/recipe-versions/{id}/compare/{other_id}` | Compares two versions (of the same recipe or of two recipes): header fields, lines by name, and the calculation summaries. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/recipe-versions/{id}/publish` | Publishes a draft: refreshes the equipment snapshot, calculates, stores the summary and makes the version immutable. Home mode needs `recipe.edit`; Business mode needs `recipe.approve`. TODO: the review steps of Business mode (submit, review, reject) come in stage 6. | recipe.edit | required | required |
| POST | `/v1/workspaces/{workspace_id}/recipe-versions/{id}/scale` | Previews a recipe scaled by a factor or to a new batch volume. Nothing is saved: send the lines in a patch to a draft to keep them. TODO: rounding of small amounts comes with the scaling screen (stage 3). | recipe.view |  |  |
| GET | `/v1/workspaces/{workspace_id}/recipes` | Lists the recipes of the workspace. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/recipes` | Creates a recipe with its first draft. | recipe.edit | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/recipes/{id}` | Deletes a recipe (soft delete). Batches that use its versions keep their data. | recipe.edit |  | required |
| GET | `/v1/workspaces/{workspace_id}/recipes/{id}` | Reads a recipe. | recipe.view |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/recipes/{id}` | Changes the name, folder, notes or lock of a recipe. | recipe.edit |  | required |
| DELETE | `/v1/workspaces/{workspace_id}/recipes/{id}/edit-session` | Releases the edit session of the caller. The call is safe to repeat. | recipe.edit |  |  |
| GET | `/v1/workspaces/{workspace_id}/recipes/{id}/edit-session` | Reads the edit session (soft lock) of a recipe. The answer is `null` when nobody holds it. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/recipes/{id}/edit-session` | Starts an edit session (takes the soft lock). Another user who holds it gets `409` with the holder in `current`, unless `take_over` is true. | recipe.edit |  |  |
| PUT | `/v1/workspaces/{workspace_id}/recipes/{id}/edit-session` | Extends the edit session of the caller (heartbeat). Use it about every 30 seconds. | recipe.edit |  |  |
| DELETE | `/v1/workspaces/{workspace_id}/recipes/{id}/favorite` | Removes the favorite mark of the caller. The call is safe to repeat. | recipe.view |  |  |
| PUT | `/v1/workspaces/{workspace_id}/recipes/{id}/favorite` | Marks a recipe as favorite for the caller. The call is safe to repeat. | recipe.view |  |  |
| DELETE | `/v1/workspaces/{workspace_id}/recipes/{id}/tags/{tag_id}` | Removes a tag from a recipe. The call is safe to repeat. | recipe.edit |  |  |
| PUT | `/v1/workspaces/{workspace_id}/recipes/{id}/tags/{tag_id}` | Puts a tag on a recipe. The call is safe to repeat. | recipe.edit |  |  |
| GET | `/v1/workspaces/{workspace_id}/recipes/{id}/versions` | Lists the versions of a recipe, newest first (header only, no lines). | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/recipes/{id}/versions` | Starts a new draft that copies a published version. | recipe.edit | required |  |
| GET | `/v1/workspaces/{workspace_id}/tags` | Lists the tags of the workspace. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/tags` | Creates a tag. | recipe.edit | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/tags/{id}` | Deletes a tag (soft delete) and removes it from the recipes. | recipe.edit |  | optional |
| PATCH | `/v1/workspaces/{workspace_id}/tags/{id}` | Renames a tag or changes its color. | recipe.edit |  | optional |

## search

Global search over recipes, batches, ingredients and stock

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/workspaces/{workspace_id}/search` | Searches recipes, batches, ingredients and stock of the workspace. | recipe.view |  |  |

## settings

Equipment, profiles, vessels, workspace and user settings (last writer wins for each field, with a change history)

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/workspaces/{workspace_id}/carbonation-profiles` | Lists records: Carbonation profile: method and target CO2 volumes | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/carbonation-profiles` | Creates a record: Carbonation profile: method and target CO2 volumes | settings.equipment | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/carbonation-profiles/{id}` | Deletes a record (soft delete; the history can restore it) | settings.equipment |  | optional |
| GET | `/v1/workspaces/{workspace_id}/carbonation-profiles/{id}` | Reads one record | recipe.view |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/carbonation-profiles/{id}` | Changes fields of a record (merge patch). Last writer wins for each field; `If-Match` is optional | settings.equipment |  | optional |
| GET | `/v1/workspaces/{workspace_id}/carbonation-profiles/{id}/history` | Change history of a record, newest first. `previous_value` holds the replaced values of the changed fields | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/carbonation-profiles/{id}/history/{history_id}/restore` | Writes the values of a history row back (a new change with its own history row) | settings.equipment |  |  |
| GET | `/v1/workspaces/{workspace_id}/equipment` | Lists records: Equipment profile: volumes, losses, efficiency, altitude and hop utilization | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/equipment` | Creates a record: Equipment profile: volumes, losses, efficiency, altitude and hop utilization | settings.equipment | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/equipment/{id}` | Deletes a record (soft delete; the history can restore it) | settings.equipment |  | optional |
| GET | `/v1/workspaces/{workspace_id}/equipment/{id}` | Reads one record | recipe.view |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/equipment/{id}` | Changes fields of a record (merge patch). Last writer wins for each field; `If-Match` is optional | settings.equipment |  | optional |
| POST | `/v1/workspaces/{workspace_id}/equipment/{id}/calibration-proposal` | Proposes a new efficiency from measured results of past batches (the engine calculates the yields). Changes nothing | settings.equipment |  |  |
| GET | `/v1/workspaces/{workspace_id}/equipment/{id}/history` | Change history of a record, newest first. `previous_value` holds the replaced values of the changed fields | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/equipment/{id}/history/{history_id}/restore` | Writes the values of a history row back (a new change with its own history row) | settings.equipment |  |  |
| POST | `/v1/workspaces/{workspace_id}/equipment/wizard` | Creates an equipment record from the four answers of the equipment wizard | settings.equipment | required |  |
| GET | `/v1/workspaces/{workspace_id}/fermentation-profiles` | Lists records: Fermentation profile with its temperature steps | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/fermentation-profiles` | Creates a record: Fermentation profile with its temperature steps | settings.equipment | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/fermentation-profiles/{id}` | Deletes a record (soft delete; the history can restore it) | settings.equipment |  | optional |
| GET | `/v1/workspaces/{workspace_id}/fermentation-profiles/{id}` | Reads one record | recipe.view |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/fermentation-profiles/{id}` | Changes fields of a record (merge patch). Last writer wins for each field; `If-Match` is optional | settings.equipment |  | optional |
| GET | `/v1/workspaces/{workspace_id}/fermentation-profiles/{id}/history` | Change history of a record, newest first. `previous_value` holds the replaced values of the changed fields | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/fermentation-profiles/{id}/history/{history_id}/restore` | Writes the values of a history row back (a new change with its own history row) | settings.equipment |  |  |
| GET | `/v1/workspaces/{workspace_id}/mash-profiles` | Lists records: Mash profile with its steps | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/mash-profiles` | Creates a record: Mash profile with its steps | settings.equipment | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/mash-profiles/{id}` | Deletes a record (soft delete; the history can restore it) | settings.equipment |  | optional |
| GET | `/v1/workspaces/{workspace_id}/mash-profiles/{id}` | Reads one record | recipe.view |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/mash-profiles/{id}` | Changes fields of a record (merge patch). Last writer wins for each field; `If-Match` is optional | settings.equipment |  | optional |
| GET | `/v1/workspaces/{workspace_id}/mash-profiles/{id}/history` | Change history of a record, newest first. `previous_value` holds the replaced values of the changed fields | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/mash-profiles/{id}/history/{history_id}/restore` | Writes the values of a history row back (a new change with its own history row) | settings.equipment |  |  |
| GET | `/v1/workspaces/{workspace_id}/me/settings` | Reads the user settings of the caller |  |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/me/settings` | Changes keys of the user settings of the caller (merge patch; `null` removes a key). Last writer wins for each key |  |  | optional |
| GET | `/v1/workspaces/{workspace_id}/me/settings/history` | Change history of the user settings of the caller, newest first |  |  |  |
| POST | `/v1/workspaces/{workspace_id}/me/settings/history/{history_id}/restore` | Writes the keys of a history row back into the user settings of the caller |  |  |  |
| GET | `/v1/workspaces/{workspace_id}/settings` | Reads the workspace settings |  |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/settings` | Changes keys of the workspace settings (merge patch; `null` removes a key). Last writer wins for each key | workspace.manage |  | optional |
| GET | `/v1/workspaces/{workspace_id}/settings/history` | Change history of the workspace settings, newest first |  |  |  |
| POST | `/v1/workspaces/{workspace_id}/settings/history/{history_id}/restore` | Writes the keys of a history row back into the workspace settings | workspace.manage |  |  |
| GET | `/v1/workspaces/{workspace_id}/vessels` | Lists records: Vessel (fermenter, kettle, keg store, ...) as Home mode uses it | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/vessels` | Creates a record: Vessel (fermenter, kettle, keg store, ...) as Home mode uses it | settings.equipment | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/vessels/{id}` | Deletes a record (soft delete; the history can restore it) | settings.equipment |  | optional |
| GET | `/v1/workspaces/{workspace_id}/vessels/{id}` | Reads one record | recipe.view |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/vessels/{id}` | Changes fields of a record (merge patch). Last writer wins for each field; `If-Match` is optional | settings.equipment |  | optional |
| GET | `/v1/workspaces/{workspace_id}/vessels/{id}/history` | Change history of a record, newest first. `previous_value` holds the replaced values of the changed fields | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/vessels/{id}/history/{history_id}/restore` | Writes the values of a history row back (a new change with its own history row) | settings.equipment |  |  |
| GET | `/v1/workspaces/{workspace_id}/water-profiles` | Lists records: Source or target water profile; `source_document_id` links the lab water analysis report (a scanned document) | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/water-profiles` | Creates a record: Source or target water profile; `source_document_id` links the lab water analysis report (a scanned document) | settings.equipment | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/water-profiles/{id}` | Deletes a record (soft delete; the history can restore it) | settings.equipment |  | optional |
| GET | `/v1/workspaces/{workspace_id}/water-profiles/{id}` | Reads one record | recipe.view |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/water-profiles/{id}` | Changes fields of a record (merge patch). Last writer wins for each field; `If-Match` is optional | settings.equipment |  | optional |
| GET | `/v1/workspaces/{workspace_id}/water-profiles/{id}/history` | Change history of a record, newest first. `previous_value` holds the replaced values of the changed fields | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/water-profiles/{id}/history/{history_id}/restore` | Writes the values of a history row back (a new change with its own history row) | settings.equipment |  |  |

## shared-devices

Shared device mode: devices, member PINs, user switching

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| POST | `/v1/shared-devices/{id}/lock` | Locks the device: the signed-in member's session ends at once. |  |  |  |
| GET | `/v1/shared-devices/{id}/members` | The tiles of the device: members who have a PIN. |  |  |  |
| POST | `/v1/shared-devices/{id}/switch` | A member steps up to the device: PIN in, user session out. |  |  |  |
| PUT | `/v1/workspaces/{ws}/me/pin` | Sets (or replaces) the caller's own PIN. Replacing clears a lock. |  |  |  |
| DELETE | `/v1/workspaces/{ws}/members/{member}/pin` | Clears a member's PIN (forgotten PIN, lock): the member sets a new one. | devices.manage |  |  |
| GET | `/v1/workspaces/{ws}/shared-devices` | Lists the workspace's shared devices. | devices.manage |  |  |
| POST | `/v1/workspaces/{ws}/shared-devices` | Opens shared device mode for a device. Needs `devices.manage` and the second step. | devices.manage; step-up |  |  |
| DELETE | `/v1/workspaces/{ws}/shared-devices/{id}` | Revokes a device: its key stops working and its sessions close. | devices.manage |  |  |
| POST | `/v1/workspaces/{ws}/shared-devices/{id}/rotate-key` | Replaces the device key. Needs `devices.manage` and the second step; the device's open session closes. | devices.manage; step-up |  |  |

## stock

Stock lots, the movement ledger, goods receipt, count, deduction for a batch and the shopping list

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| POST | `/v1/workspaces/{workspace_id}/batches/{id}/stock-deduction` | Deducts the actual amounts of a batch from stock. Each ingredient is taken from the oldest lots first (best-before day, then receipt day); a lot that the actual record names comes first. The call is safe to repeat: it deducts only what the earlier calls did not deduct. A shortage goes to the last lot, which then shows a `negative_stock` warning. | stock.move | required |  |
| GET | `/v1/workspaces/{workspace_id}/shopping-list` | Lists the shopping list. | recipe.view |  |  |
| DELETE | `/v1/workspaces/{workspace_id}/shopping-list/{id}` | Removes an item from the list (soft delete). | stock.move |  | required |
| PATCH | `/v1/workspaces/{workspace_id}/shopping-list/{id}` | Ticks off an item or changes its amount. | stock.move |  | required |
| POST | `/v1/workspaces/{workspace_id}/shopping-list/from-recipe` | Builds shopping list items from a recipe version: the amounts of the recipe (scaled to the batch volume) minus the stock on hand. An item for the same ingredient, unit and recipe version that is still open gets a new amount, so the call is safe to repeat. | stock.move | required |  |
| GET | `/v1/workspaces/{workspace_id}/stock/lots` | Lists the stock lots of the workspace. | recipe.view |  |  |
| GET | `/v1/workspaces/{workspace_id}/stock/lots/{id}` | Reads one lot. | recipe.view |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/stock/lots/{id}` | Changes the supplier, best-before day, cost or notes of a lot. The amount changes only with a movement. | stock.move |  | required |
| POST | `/v1/workspaces/{workspace_id}/stock/lots/{id}/movements` | Writes a movement on a lot: `receive`, `consume`, `adjust` or `count`. The answer carries a `negative_stock` warning when the lot is below zero afterwards. | stock.move | required |  |
| GET | `/v1/workspaces/{workspace_id}/stock/movements` | Lists the ledger rows. | recipe.view |  |  |
| POST | `/v1/workspaces/{workspace_id}/stock/receipts` | Receives goods: finds or creates the lot (ingredient and lot code) and writes a `receive` row. | stock.move | required |  |
| GET | `/v1/workspaces/{workspace_id}/stock/totals` | Totals for each ingredient and unit, calculated from the ledger rows. An ingredient below zero gets a `negative_stock` warning. | recipe.view |  |  |

## subscription

Trial, entitlement and subscriptions

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/workspaces/{ws}/subscription` | Subscription state of the workspace (trial, entitlement, subscriptions). |  |  |  |

## support

Support requests of the customer

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/workspaces/{workspace_id}/support/access-history` | The grants and the operations of the support team in the workspace. | support.grant |  |  |
| POST | `/v1/workspaces/{workspace_id}/support/grants` | Gives support access to the content of the workspace. | support.grant | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/support/grants/{id}` | Revokes a grant. The access stops at once. | support.grant |  |  |
| GET | `/v1/workspaces/{workspace_id}/support/requests` | The requests that the caller opened, newest first. |  |  |  |
| POST | `/v1/workspaces/{workspace_id}/support/requests` | Opens a support request with its first message. |  | required |  |
| GET | `/v1/workspaces/{workspace_id}/support/requests/{id}` | A request with its messages. |  |  |  |
| POST | `/v1/workspaces/{workspace_id}/support/requests/{id}/messages` | Adds a message. A request that was answered or resolved opens again. A closed request takes no message: the customer opens a new request that continues the old one. |  | required |  |
| DELETE | `/v1/workspaces/{workspace_id}/support/requests/{id}/messages/{message_id}` | Retracts a message of the caller. The comment in the Issue is deleted. |  |  |  |
| PATCH | `/v1/workspaces/{workspace_id}/support/requests/{id}/messages/{message_id}` | Edits a message of the caller. The comment in the Issue changes too. |  |  |  |
| POST | `/v1/workspaces/{workspace_id}/support/requests/{id}/resolve` | The customer marks a request as resolved. The Issue closes. |  |  |  |

## system

Service status and version

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/healthz` | Liveness probe (unversioned). |  |  |  |
| GET | `/readyz` | Readiness probe (unversioned): the database must answer. |  |  |  |
| GET | `/v1/version` | Reports the API and engine versions. |  |  |  |

## trash

Recently deleted records (30 days) and their restore

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| GET | `/v1/workspaces/{workspace_id}/trash` | Lists the records deleted in the last 30 days that the caller may restore. |  |  |  |
| POST | `/v1/workspaces/{workspace_id}/trash/{kind}/{id}/restore` | Restores a deleted record within 30 days of its deletion. |  |  |  |

## workspaces

Workspaces

| Yöntem | Yol | Açıklama | Yetki / ek adım | Idempotency-Key | If-Match |
|---|---|---|---|---|---|
| POST | `/v1/workspaces` | Opens a workspace; the caller becomes its first member with the administrator role. |  |  |  |

## Ortak hata kodları

OpenAPI ErrorCode enum:

- `malformed_request`
- `unauthenticated`
- `forbidden`
- `step_up_required`
- `entitlement_required`
- `not_found`
- `conflict`
- `idempotency_in_progress`
- `version_mismatch`
- `payload_too_large`
- `unsupported_media_type`
- `validation_failed`
- `idempotency_mismatch`
- `calculation_failed`
- `upgrade_required`
- `precondition_required`
- `rate_limited`
- `internal`
- `unavailable`

Hata gövdesi RFC 9457 ProblemDetails biçimindedir. Spec'te tanımlanan alanlar: type, title, status, code; isteğe bağlı current, errors ve request_id. request_id açıklaması X-Request-Id yanıt başlığını işaret eder.

## Ortak ve operasyonel başlıklar

| Başlık | Kullanım |
|---|---|
| `Idempotency-Key` | Client generated key (ADR-0008 rule 6) (required) |
| `If-Match` | Version `ETag` of the record (optional) (required) |
| `If-None-Match` | Version `ETag` the client holds (optional) |
| `Last-Event-ID` | Last event id the client saw; the server replays what followed (one hour) (optional) |
| `X-Device-Key` | The device key (required) |
| `X-Request-Id` | ProblemDetails.request_id açıklamasında yanıt korelasyon kimliği olarak belirtilir. |
