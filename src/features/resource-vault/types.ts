export interface VaultResource {
  id: string; publicNo: number; title: string; slug: string; summary: string; content: string; category: string; tags: string[]
  coverUrl: string; panType: string; obtainUrl: string; obtainCode: string; obtainNote: string; price: number; isFeatured: boolean
  viewCount: number; purchaseCount: number; ownerName: string; createdAt: string; updatedAt: string; publishedAt: string; canViewObtain: boolean
}

