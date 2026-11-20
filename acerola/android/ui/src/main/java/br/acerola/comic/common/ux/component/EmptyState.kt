package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Inbox
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.ui.R

/**
 * Acerola standard empty state: the mascot (or a round icon, for small empty states inside a
 * card) + a Black title + one phrase + one action. The mascot is a local drawable: the app is
 * open source and does not depend on the brand CDN.
 */
@Composable
fun Acerola.Component.EmptyState(
    title: String,
    modifier: Modifier = Modifier,
    hint: String? = null,
    icon: ImageVector? = null,
    action: (@Composable () -> Unit)? = null,
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier = modifier.widthIn(max = 360.dp).fillMaxWidth().padding(vertical = SpacingTokens.Large),
    ) {
        if (icon != null) {
            Box(
                contentAlignment = Alignment.Center,
                modifier =
                    Modifier
                        .size(56.dp)
                        .clip(AcerolaShapes.Full)
                        .background(MaterialTheme.colorScheme.surfaceVariant),
            ) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(24.dp),
                )
            }
        } else {
            Image(
                painter = painterResource(id = R.drawable.acerola_mascot),
                contentDescription = null,
                modifier = Modifier.size(120.dp),
            )
        }

        Spacer(modifier = Modifier.height(SpacingTokens.Small))

        Text(
            text = title,
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onBackground,
            textAlign = TextAlign.Center,
        )

        if (hint != null) {
            Spacer(modifier = Modifier.height(SpacingTokens.ExtraSmall))
            Text(
                text = hint,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
            )
        }

        if (action != null) {
            Spacer(modifier = Modifier.height(SpacingTokens.Large))
            action()
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun EmptyStatePreview() {
    AcerolaTheme {
        Acerola.Component.EmptyState(title = "Sua biblioteca está vazia", hint = "Escolha uma pasta com seus quadrinhos.")
    }
}

@Preview(name = "Icon", showBackground = true)
@Composable
private fun EmptyStateIconPreview() {
    AcerolaTheme {
        Acerola.Component.EmptyState(title = "Nenhum aparelho", hint = "Pareie um aparelho na mesma rede.", icon = Icons.Rounded.Inbox)
    }
}
