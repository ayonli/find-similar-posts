import { abortWith } from "@ayonli/jsext/async"
// @deno-types="./index.d.ts"
import {
    type DbOptions,
    type IssueFeatures,
    type IssueFeaturesRecord,
    IssueFeatureStore as IssueFeatureStoreNative,
    type SimilarIssueFeaturesRecord,
} from "./index.js"

export type { DbOptions, IssueFeatures, IssueFeaturesRecord, SimilarIssueFeaturesRecord }

export class IssueFeatureStore {
    #impl: IssueFeatureStoreNative

    static async fromDb(options: DbOptions): Promise<IssueFeatureStore> {
        const impl = await IssueFeatureStoreNative.fromDb(options)
        const ins = new this()
        ins.#impl = impl
        return ins
    }

    constructor(records?: IssueFeaturesRecord[] | null | undefined) {
        this.#impl = new IssueFeatureStoreNative(records)
    }

    setRecord(record: IssueFeaturesRecord): void {
        this.#impl.setRecord(record)
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
