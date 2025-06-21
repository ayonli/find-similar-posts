import { abortWith } from "@ayonli/jsext/async"
// @deno-types="./index.d.ts"
import {
    type IssueFeatures,
    type IssueFeaturesRecord,
    IssueFeatureStore as IssueFeatureStoreNative,
    type SimilarIssueFeaturesRecord,
} from "./index.js"

export type { IssueFeatures, IssueFeaturesRecord, SimilarIssueFeaturesRecord }

export class IssueFeatureStore {
    #impl = new IssueFeatureStoreNative()

    preload(records: IssueFeaturesRecord[]): void {
        this.#impl.preload(records)
    }

    addRecord(record: IssueFeaturesRecord): void {
        this.#impl.addRecord(record)
    }

    getRecord(issueId: string): IssueFeaturesRecord | null {
        return this.#impl.getRecord(issueId)
    }

    removeRecord(issueId: string): boolean {
        return this.#impl.removeRecord(issueId)
    }

    async findSimilarRecords(
        features: IssueFeatures,
        options: {
            topN?: number
            signal?: AbortSignal | null
        } = {},
    ): Promise<SimilarIssueFeaturesRecord[]> {
        // NAPI-RS has a bug when reusing the same AbortSignal, so we derive a
        // new one from the parent instead.
        const signal = options.signal ? abortWith(options.signal).signal : undefined
        return await this.#impl.findSimilarRecords(
            features,
            options.topN,
            signal,
        )
    }
}
