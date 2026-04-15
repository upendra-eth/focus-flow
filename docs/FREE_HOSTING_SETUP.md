# FocusFlow — Free Hosting Setup (5 Minutes)

## Architecture: 100% Free Stack

```
Android Phone (APK)
       │
       ▼
Render.com (free)          ← Rust API server
       │
       ├──▶ Neon.tech (free)    ← PostgreSQL database
       └──▶ Upstash.com (free)  ← Redis cache
```

**Total monthly cost: $0** (+ ~$0.70/month OpenAI API)

## Step 1: Create Free PostgreSQL (Neon.tech)

1. Go to **https://neon.tech** → Sign up (free, no credit card)
2. Create a new project → name it `focusflow`
3. Copy the connection string. It looks like:
   ```
   postgres://focusflow_owner:AbCdEf123@ep-cool-name-123456.us-east-2.aws.neon.tech/focusflow?sslmode=require
   ```
4. In the Neon dashboard SQL Editor, paste and run the contents of:
   - `migrations/001_initial_schema.sql`
   - `migrations/002_seed_profiling_questions.sql`

That's it — database is ready.

## Step 2: Create Free Redis (Upstash.com)

1. Go to **https://upstash.com** → Sign up (free, no credit card)
2. Create a new Redis database → name it `focusflow`
3. Copy the connection string. It looks like:
   ```
   rediss://default:AbCdEf123@us1-great-moose-12345.upstash.io:6379
   ```

## Step 3: Deploy Backend to Render.com

1. **Push your code to GitHub:**
   ```bash
   cd /Users/upendrasingh/data/ai-assistance/focusflow
   git init
   git add .
   git commit -m "Initial FocusFlow commit"
   # Create a repo on GitHub, then:
   git remote add origin https://github.com/YOUR_USERNAME/focusflow.git
   git push -u origin main
   ```

2. Go to **https://render.com** → Sign up (free, no credit card)

3. Click **New** → **Web Service**

4. Connect your GitHub repo → select `focusflow`

5. Configure:
   - **Name**: `focusflow-api`
   - **Root Directory**: (leave empty)
   - **Runtime**: Docker
   - **Dockerfile Path**: `backend/Dockerfile`
   - **Docker Context**: `.`
   - **Instance Type**: Free

6. Add **Environment Variables**:
   | Key | Value |
   |-----|-------|
   | `DATABASE_URL` | (paste Neon connection string from Step 1) |
   | `REDIS_URL` | (paste Upstash connection string from Step 2) |
   | `OPENAI_API_KEY` | `sk-your-openai-key` |
   | `JWT_SECRET` | (click "Generate" or type any random string) |
   | `RUST_LOG` | `focusflow_api=info` |
   | `SERVER_HOST` | `0.0.0.0` |
   | `SERVER_PORT` | `10000` |

7. Click **Create Web Service**

8. Wait ~5 minutes for the Docker image to build

9. Your API will be live at: `https://focusflow-api.onrender.com`

## Step 4: Update the APK

Before building the APK, update the API URL in the Android app:

Edit `android/app/build.gradle.kts`:
```kotlin
buildConfigField("String", "API_BASE_URL", "\"https://focusflow-api.onrender.com\"")
```

Then rebuild the APK.

## Free Tier Limits (What to Know)

### Render.com Free Tier
- **Cold starts**: Server sleeps after 15 min of inactivity. First request after sleep takes ~30 seconds.
- **750 hours/month**: Enough for 24/7 (720 hours in a month).
- **512 MB RAM**: Plenty for our Rust binary (~20MB memory).
- **Workaround for cold starts**: The Android app works offline, so cold starts only matter when syncing.

### Neon.tech Free Tier
- **0.5 GB storage**: Enough for thousands of tasks/answers.
- **Auto-suspend**: Database pauses after 5 min inactivity. Wakes in ~1 second.
- **191 compute hours/month**: Plenty for personal use.

### Upstash Free Tier
- **10,000 commands/day**: Enough for personal use.
- **256 MB storage**: Way more than needed for widget cache.
- **Always on**: No cold starts.

### OpenAI API
- **Pay-as-you-go**: Add $5 credit (lasts months for personal use).
- **~$0.70/month** for ~100 voice classifications + 4 weekly insights.

## Quick Test After Deploy

```bash
# Test the deployed API
curl -s -X POST https://focusflow-api.onrender.com/api/v1/auth/device \
  -H "Content-Type: application/json" \
  -d '{"device_id": "my-phone"}' | python3 -m json.tool
```

## Share the APK

After building the APK:
```
android/app/build/outputs/apk/debug/app-debug.apk
```

Send this file to anyone via:
- AirDrop / email / WhatsApp / Google Drive
- They tap it → "Install" → allow unknown sources → done

The APK is ~5-10 MB. It connects to your Render.com backend automatically.
